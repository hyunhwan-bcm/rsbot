//! Port of PyBot's `bot/knowledge/wording.py`: what to say - the card, the note, the message
//! to a landlord, a boss or a friend - and the one thing to keep in mind about tone.

use crate::data::{self, FnRef};
use crate::knowledge::d1_dup_g5 as dup;
use crate::knowledge::d1_shared::{self as sh, ShapeFn};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as re, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.knowledge.wording.";

fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("{M}{name}"))
}
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn string(name: &str) -> String {
    data::string(&format!("{M}{name}"))
}

static A_CHILD_NAMED: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_CHILD_NAMED"));
static ONLY_SAID_ABOUT_A_CHILD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ONLY_SAID_ABOUT_A_CHILD"));
static SOMEBODY_AT_HOME: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SOMEBODY_AT_HOME"));
static THEIR_SCREENS: LazyLock<&'static Re> = LazyLock::new(|| rx("_THEIR_SCREENS"));
static MY_OWN_SCREENS: LazyLock<&'static Re> = LazyLock::new(|| rx("_MY_OWN_SCREENS"));
static NO_FUSS: LazyLock<&'static Re> = LazyLock::new(|| rx("_NO_FUSS"));
static A_COLLEAGUE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_COLLEAGUE"));
static A_CLOSE_ONE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_CLOSE_ONE"));
static ASKS_FOR_THE_WORDS: LazyLock<&'static Re> = LazyLock::new(|| rx("_ASKS_FOR_THE_WORDS"));

static TOKENS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static AGE_ORDINAL: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\d{1,3})(?:st|nd|rd|th)\b", 0));
static AGE_AFTER: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"(?:birthday|turning|turns|going on|hits?)\D{0,12}(\d{1,3})\b", 0));
static AGE_WHEN: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"\b(\d{2,3})\b\s*(?:next|this|in)\s*",
            r"(?:week|month|year|weekend|june|july|august|september|october|november",
            r"|december|january|february|march|april|may|christmas|summer|spring",
            r"|autumn|winter|easter)\b"
        ),
        0,
    )
});
static AGE_YEARS_OLD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\d{1,2})\s*(?:year|yr)s?[ -]?old\b", 0));

// --- whose screens ------------------------------------------------------------------------------

/// The line as one lower-case string with spaces round it, or "" when there is no line.
pub fn the_line_said(understood: &Understood) -> String {
    let said = text::squash(sh::clean_or_text(understood)).to_lowercase();
    if said.is_empty() {
        String::new()
    } else {
        format!(" {said} ")
    }
}

/// Whether the line names one of these phrases (a phrase as text, a single word as a token).
pub fn _names_one_of<S: AsRef<str>>(said: &str, phrases: &[S]) -> bool {
    let tokens: HashSet<String> = TOKENS.findall_str(said).into_iter().collect();
    phrases.iter().any(|p| {
        let p = p.as_ref();
        if p.contains(' ') {
            said.contains(p)
        } else {
            tokens.contains(p)
        }
    })
}

/// True when the screens in the line belong to a child, so a parent's limit answer fits.
pub fn screens_are_a_childs(understood: &Understood) -> bool {
    let said = the_line_said(understood);
    if text::strip(&said).is_empty() {
        return true;
    }
    if sh::any_in(&said, &ONLY_SAID_ABOUT_A_CHILD) {
        return true;
    }
    _names_one_of(&said, &A_CHILD_NAMED)
}

/// True when the phone in the line is not the writer's own (nor a child's).
pub fn screens_are_somebody_elses(understood: &Understood) -> bool {
    let said = the_line_said(understood);
    if text::strip(&said).is_empty() {
        return false;
    }
    if MY_OWN_SCREENS.is_match(&said) {
        return false;
    }
    if screens_are_a_childs(understood) {
        return false;
    }
    if THEIR_SCREENS.is_match(&said) {
        return true;
    }
    SOMEBODY_AT_HOME.iter().any(|p| said.contains(&format!("{p} ")))
}

// --- the birthday card ---------------------------------------------------------------------------

/// The age the line names (an ordinal, a number after "turning", "90 next month", "6 year old"),
/// or None when it names none between 1 and 120.
pub fn _birthday_age(text: &str) -> Option<i64> {
    let said = text.to_lowercase();
    let found = AGE_ORDINAL
        .search(&said)
        .or_else(|| AGE_AFTER.search(&said))
        .or_else(|| AGE_WHEN.search(&said))
        .or_else(|| AGE_YEARS_OLD.search(&said))?;
    let age = dup::py_int(found.str(1))?;
    if (1..=120).contains(&age) {
        Some(age)
    } else {
        None
    }
}

/// "1st", "2nd", "11th", "23rd": the number as an ordinal.
pub fn _ordinal(number: i64) -> String {
    if [11, 12, 13].contains(&number.rem_euclid(100)) {
        return format!("{number}th");
    }
    let suffix = match number.rem_euclid(10) {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    };
    format!("{number}{suffix}")
}

/// A shape: the line gives the age the card is for, which is what picks its example.
pub fn _names_an_age(words: &str) -> bool {
    _birthday_age(words).is_some()
}

/// The words for a birthday card, with the age the person gave rather than a made-up one.
pub fn _card_birthday(_memory: &Memory, reading: &Understood) -> String {
    let said = sh::clean_or_text(reading);
    let age = _birthday_age(said);
    let which = match age {
        Some(a) => _ordinal(a),
        None => "birthday".to_string(),
    };
    if NO_FUSS.is_match(said) {
        return format!(
            "Keep it plain and quiet, because that is the card a person who hates a fuss \
             actually wants: \"Happy {which} - no fuss, I'm just glad you're in my life.\" One \
             true thing about them, no joke and no speech, and give the present quietly \
             rather than in front of everyone."
        );
    }
    let example = if age.is_some_and(|a| a < 13) {
        format!(
            "\"Happy {which} - the year you finally beat me at cards.\" One true thing they did \
             beats any joke, and for a small child the card is really for the parent: keep \
             it short and kind."
        )
    } else if age.is_some_and(|a| a < 18) {
        format!(
            "\"Happy {which} - still the best person to be stuck in a car with.\" One thing the \
             two of you actually do together beats any joke about the number, and a \
             teenager reads a short card: no advice, no speech, and the money is welcome."
        )
    } else if A_COLLEAGUE.is_match(said) {
        format!(
            "\"Happy {which} - hope you get spoiled.\" With someone you do not know well the \
             warm plain line is the card itself: no in-joke, and put the money or gift \
             honestly (\"Have a drink on me\")."
        )
    } else if A_CLOSE_ONE.is_match(said) {
        format!(
            "\"Happy {which} - I still tell people about the night you drove us home at 3am.\" \
             They are close enough for the in-joke, so use the one only the two of you \
             would get."
        )
    } else {
        format!(
            "\"Happy {which} - I still tell people about the night you drove us home at 3am.\" \
             For a colleague or someone you do not know well, a warm plain line is enough \
             (\"Happy birthday - hope you get spoiled\"), and put the money or gift \
             honestly (\"Have a drink on me\")."
        )
    };
    format!("Name one thing you like about them and skip the round-number joke: {example}")
}

/// The card for an occasion the line never names, and the quiet one when it asks for one.
pub fn _card_plain(_memory: &Memory, reading: &Understood) -> String {
    let said = sh::clean_or_text(reading);
    if NO_FUSS.is_match(said) {
        return "Keep it plain and quiet, because that is the card this person actually wants: \
                \"Happy birthday - no fuss, I'm just glad you're in my life.\" One true thing \
                about them, no joke and no speech, and give the present quietly rather than in \
                front of everyone."
            .to_string();
    }
    "Name one real thing about them and one wish, then stop - two lines is a card: \
     \"Thinking of you both - hope the year is a good one.\" If you do not know them well, \
     warm and plain beats clever, and sign it with your full name."
        .to_string()
}

/// A shape: the line asks what words to use, whatever modal carries the ask.
pub fn _asks_for_the_words(line: &str) -> bool {
    ASKS_FOR_THE_WORDS.is_match(&line.to_lowercase())
}

// --- raising a problem with a friend ---------------------------------------------------------------

static RAISE_EXAMPLES: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.wording._RAISE_EXAMPLES")
        .items()
        .iter()
        .filter_map(|row| {
            let row = row.items();
            Some((row.first()?.as_re()?, row.get(1)?.as_str()?.to_string()))
        })
        .collect()
});
static RAISE_ASK_SUBJECTS: LazyLock<Vec<&'static Re>> = LazyLock::new(|| {
    data::get("bot.knowledge.wording._RAISE_ASK_SUBJECTS").items().iter().filter_map(|v| v.as_re()).collect()
});
static RAISE_PLAIN: LazyLock<String> = LazyLock::new(|| string("_RAISE_PLAIN"));
static SOMEONE_HURT_A_PART: LazyLock<&'static Re> = LazyLock::new(|| rx("_SOMEONE_HURT_A_PART"));
static TALKS_OVER: LazyLock<&'static Re> = LazyLock::new(|| rx("_TALKS_OVER"));
static MOVING_OUT: LazyLock<&'static Re> = LazyLock::new(|| rx("_MOVING_OUT"));
static MOVING_AWAY: LazyLock<&'static Re> = LazyLock::new(|| rx("_MOVING_AWAY"));
static BABY: LazyLock<&'static Re> = LazyLock::new(|| rx("_BABY"));
static RAISE_SMOKE: LazyLock<&'static Re> = LazyLock::new(|| rx("_RAISE_SMOKE"));
static RAISE_HOUSE: LazyLock<&'static Re> = LazyLock::new(|| rx("_RAISE_HOUSE"));
static MESS: LazyLock<&'static Re> = LazyLock::new(|| rx("_MESS"));
static SHARED_HOUSE: LazyLock<&'static Re> = LazyLock::new(|| rx("_SHARED_HOUSE"));
static MONEY: LazyLock<&'static Re> = LazyLock::new(|| rx("_MONEY"));
static SHARE_MONEY: LazyLock<&'static Re> = LazyLock::new(|| rx("_SHARE_MONEY"));
static PARENT_MONEY: LazyLock<&'static Re> = LazyLock::new(|| rx("_PARENT_MONEY"));
static HOUSEHOLD_MONEY: LazyLock<&'static Re> = LazyLock::new(|| rx("_HOUSEHOLD_MONEY"));
static SAID_IT_IS_NEWS: LazyLock<&'static Re> = LazyLock::new(|| rx("_SAID_IT_IS_NEWS"));

/// The example sentence for this line, from the subject it names (only among `patterns` when
/// given) - or None.
pub fn _raise_example(line: &str, patterns: Option<&[&Re]>) -> Option<String> {
    for (pattern, words) in RAISE_EXAMPLES.iter() {
        // `pattern not in patterns`: compiled patterns compare by pattern text and flags.
        if let Some(allowed) = patterns {
            if !allowed.iter().any(|p| p.pattern() == pattern.pattern() && p.flags() == pattern.flags()) {
                continue;
            }
        }
        if pattern.is_match(line) {
            return Some(words.clone());
        }
    }
    None
}

/// False for a line whose "hurt" is on a body, which the body tables answer better.
pub fn _not_a_hurt_body(understood: &Understood) -> bool {
    !SOMEONE_HURT_A_PART.is_match(sh::clean_or_text(understood))
}

/// The same advice for every problem between friends, with an example that fits this one.
pub fn _raise_problem(_memory: &Memory, understood: &Understood) -> String {
    let line = sh::clean_or_text(understood);
    let example = _raise_example(line, None).unwrap_or_else(|| RAISE_PLAIN.clone());
    format!(
        "Say what happened, how it made you feel and what you want to change, once: {example}. \
         One topic at a time, no list of past grievances, and ask for their side."
    )
}

/// How to say one specific piece of news to the people it concerns; None when the line is not
/// news (an apology, a declaration) or is money with a parent.
pub fn _saying_it(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = sh::clean_or_text(understood);
    if TALKS_OVER.is_match(line) {
        return Some(
            "Say it once, plainly, and not in the moment it is happening: \"Can I finish this \
             one? You jump in and I lose the thread.\" Name what they do rather than what they \
             are - one line, no list of past times - and give them the benefit of the doubt \
             once, because most people who talk over you are excited or nervous rather than \
             rude. Then keep a short version for the moment itself, a hand up and \"hang on, \
             let me finish this bit\", since that is the only place the habit can change. If \
             it carries on after you have said it plainly, say it plainly a second time and \
             say how it lands; that is not nagging, it is the second half of the same \
             sentence."
                .to_string(),
        );
    }
    if MOVING_OUT.is_match(line) {
        return Some(
            "Give them the plan, not a request for permission: \"I've found a flat and I'm \
             moving out on the 14th - I wanted you to hear it from me first.\" Then let them \
             react without defending it, and come back to the practical bits - how often you \
             will visit, what you need - once the first conversation has cooled."
                .to_string(),
        );
    }
    if MOVING_AWAY.is_match(line) {
        return Some(
            "Tell the people you will miss before the ones who will hear it second hand, and \
             give them a date: \"I'm moving to Bristol in March - I didn't want you to find \
             out from someone else.\" Then say what you want to keep going, a call every \
             couple of weeks or a visit in the summer, so the news comes with a plan rather \
             than a goodbye."
                .to_string(),
        );
    }
    if BABY.is_match(line) {
        return Some(
            "Tell them in person or on a call rather than in a message, and lead with the news: \
             \"I'm pregnant - it's early, and I wanted you to know.\" Then say what you want \
             from them now, help or time before anyone else is told; that one sentence saves \
             the guessing."
                .to_string(),
        );
    }
    if RAISE_SMOKE.is_match(line) || RAISE_HOUSE.is_match(line) {
        let subjects: Vec<&Re> = RAISE_ASK_SUBJECTS.iter().copied().collect();
        let example = _raise_example(line, Some(&subjects)).unwrap_or_else(|| {
            "\"the music was still going at half one - could we call it midnight next time?\"".to_string()
        });
        return Some(format!(
            "Ask for the one thing, name when it happens and say what you want instead, once \
             and in private: {example} Not the whole list and not in the moment it is \
             annoying you, and leave them room to say why it is hard; a habit that is asked \
             for plainly usually bends a little, where a rule handed down does not."
        ));
    }
    if MESS.is_match(line) {
        return Some(
            "Ask for the one job rather than the whole mess, and say when: \"Could you do the \
             washing up tonight? I did it the last four nights and it is stacking up.\" Name \
             the thing, the reason and the deadline in one breath - a general complaint about \
             the flat gets an apology and no change."
                .to_string(),
        );
    }
    if SHARED_HOUSE.is_match(line) && (MONEY.is_match(line) || SHARE_MONEY.is_match(line)) {
        return Some(
            "Ask once, with the amount and the date, and leave the history out of it: \"The \
             broadband comes to 32 a month and your half is 16 - could you send it by \
             Friday?\" A plain ask reads as fair; hinting over the washing up reads as a \
             buildup."
                .to_string(),
        );
    }
    if PARENT_MONEY.is_match(line) {
        return None;
    }
    if MONEY.is_match(line) && HOUSEHOLD_MONEY.is_match(line) {
        return Some(
            "Pick a time that is not the moment a bill lands, and put the numbers side by side \
             rather than asking for a verdict: what comes in, what goes out, and what each of \
             you wants to be able to afford. Open with a question - \"how do you feel about \
             the way we split things?\" - and leave one decision for next time."
                .to_string(),
        );
    }
    if !SAID_IT_IS_NEWS.is_match(line) {
        return None;
    }
    Some(
        "Say it in one flat sentence, then stop: the news, the reason if there is one, and what \
         you want from them. The wait for the perfect moment is usually longer and worse than \
         the conversation itself."
            .to_string(),
    )
}

// --- the thank-you note ------------------------------------------------------------------------

static THANKS_FLOWERS: LazyLock<Vec<String>> = LazyLock::new(|| strs("THANKS_FLOWERS"));
static THANKS_GIFT: LazyLock<Vec<String>> = LazyLock::new(|| strs("THANKS_GIFT"));
static THANKS_STAY: LazyLock<Vec<String>> = LazyLock::new(|| strs("THANKS_STAY"));
static THANKS_MEAL: LazyLock<Vec<String>> = LazyLock::new(|| strs("THANKS_MEAL"));
static THANKS_LIFT: LazyLock<Vec<String>> = LazyLock::new(|| strs("THANKS_LIFT"));
static THANKS_FAVOUR: LazyLock<Vec<String>> = LazyLock::new(|| strs("THANKS_FAVOUR"));
static THANKS_PERSON: LazyLock<Vec<String>> = LazyLock::new(|| strs("THANKS_PERSON"));
static THANKS_ANY: LazyLock<String> = LazyLock::new(|| string("THANKS_ANY"));

/// Whether the line named one of these words (as a typed word) or phrases (in the clean line).
pub fn _thanks_has<S: AsRef<str>>(understood: &Understood, words: &[S]) -> bool {
    let typed: HashSet<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let clean = sh::clean_or_text(understood).to_lowercase();
    for word in words {
        let word = word.as_ref();
        if word.contains(' ') {
            if clean.contains(word) {
                return true;
            }
        } else if typed.contains(word) {
            return true;
        }
    }
    false
}

/// A thank-you note about the thing this line names, in two or three lines.
pub fn thanks_note(_memory: &Memory, understood: &Understood) -> String {
    if _thanks_has(understood, &THANKS_FLOWERS) {
        return "Say where the flowers are and what they have done for the room: \"Thank you for \
                the flowers - they are on the kitchen table and the whole week looks better.\" \
                Two or three lines, say when you will see them next, and stop."
            .to_string();
    }
    if _thanks_has(understood, &THANKS_GIFT) {
        return "Name the thing and what you are doing with it, rather than thanking them for it: \
                \"Thank you for the cafetiere - it is on my desk and I use it every morning.\" \
                Two or three lines, say when you will see them next, and stop; a present does not \
                need a page."
            .to_string();
    }
    if _thanks_has(understood, &THANKS_STAY) {
        return "Pick the one thing they did that you noticed - the spare bed, the early start, the \
                extra trip to the station - and say what it saved you: \"Thank you for having me \
                for the weekend - the lift on Sunday saved me the whole morning.\" Then say when \
                you will have them back, if you mean it."
            .to_string();
    }
    if _thanks_has(understood, &THANKS_MEAL) {
        return "Name the thing they did and what it changed for you, then stop: \"Thank you for \
                the meal on Saturday - I came home full and happy.\" One detail from the evening \
                that only you would know beats a paragraph of adjectives, and anything you are \
                actually going to do, say when."
            .to_string();
    }
    if _thanks_has(understood, &THANKS_LIFT) {
        return "Name the trip and what it saved you: \"Thank you for the lift to the station on \
                Friday - it saved me an hour and a soaking.\" Then one line about what it meant \
                to you, and stop."
            .to_string();
    }
    if _thanks_has(understood, &THANKS_FAVOUR) {
        return "Name the job they did and what it saved you, then stop: \"Thank you for looking \
                after the cat while we were away - coming back to a happy animal was the whole \
                holiday.\" One line about what it meant to you beats a paragraph, and say what \
                you will do in return if you mean it."
            .to_string();
    }
    if _thanks_has(understood, &THANKS_PERSON) {
        return "Say what they did and what it was worth to you: \"Thank you for the time you gave \
                me over the summer - I still use the list you wrote.\" One sentence of detail, \
                one of what it meant, then stop; people keep the short ones."
            .to_string();
    }
    THANKS_ANY.clone()
}

// --- spelling -------------------------------------------------------------------------------------

/// The forms of a table word people actually type ("separately", "separates"). Import-time in
/// PyBot (it builds `_SPELLING_TIPS`, which the bundle holds); ported for completeness.
pub fn _inflections(word: &str) -> HashSet<String> {
    let mut forms: HashSet<String> = HashSet::from([word.to_string()]);
    if !word.contains(' ') && text::len(word) > 3 {
        for end in ["ly", "s", "es", "ing", "ed"] {
            forms.insert(format!("{word}{end}"));
        }
        if let Some(stem) = word.strip_suffix('e') {
            for end in ["ing", "ed", "ly"] {
                forms.insert(format!("{stem}{end}"));
            }
        }
    }
    forms
}

static SPELLING_TIPS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.wording._SPELLING_TIPS")
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str()?.to_string())))
        .collect()
});
static SPELLING_TIP_OF: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| SPELLING_TIPS.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect());

/// Why the word this line asks about is spelled the way it is; None when it names no table word.
pub fn _spelling(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    let spaced = clean.replace('?', " ").replace(',', " ").replace(':', " ");
    for word in text::split_ref(&spaced) {
        if let Some(tip) = SPELLING_TIP_OF.get(text::strip_chars(word, ".")) {
            if !tip.is_empty() {
                return Some(tip.to_string());
            }
        }
    }
    for (phrase, tip) in SPELLING_TIPS.iter() {
        if phrase.contains(' ') && clean.contains(phrase.as_str()) {
            return Some(tip.clone());
        }
    }
    None
}

// --- the eulogy --------------------------------------------------------------------------------------

static EULOGY_LENGTH: LazyLock<Vec<String>> = LazyLock::new(|| strs("_EULOGY_LENGTH"));
static EULOGY_START: LazyLock<Vec<String>> = LazyLock::new(|| strs("_EULOGY_START"));

/// `has = lambda *wanted: any(shape in clean for shape in wanted)` inside `_eulogy`.
#[allow(non_snake_case)]
pub fn _eulogy__lambda_713<S: AsRef<str>>(clean: &str, wanted: &[S]) -> bool {
    sh::any_in(clean, wanted)
}

/// The structure, the length or the opening line of a eulogy, depending on what was asked.
pub fn _eulogy(_memory: &Memory, understood: &Understood) -> String {
    let clean = understood.clean.to_lowercase();
    if _eulogy__lambda_713(&clean, &EULOGY_LENGTH) {
        return "A eulogy runs about three to five minutes, so 500 to 750 words read slowly - time \
                it out loud once, because reading to a room is slower than reading to yourself. \
                Shorter and warm beats long and solemn: two minutes with one real story in it \
                lands better than ten minutes of a life in order, and nobody has ever wished a \
                eulogy had listed more jobs."
            .to_string();
    }
    if _eulogy__lambda_713(&clean, &EULOGY_START) {
        return "Start with a plain, specific fact about them, not a summary: \"Dad was born in \
                Leeds and never once chose a different chippy.\" Then one story that only your \
                family could tell, with a detail in it - what they said, what they were wearing, \
                the thing they did with their hands. Close on what they gave you, in one sentence, \
                and step back; the last line does not have to be poetry, it has to be theirs."
            .to_string();
    }
    "Write it in four short parts: who they were in a sentence, one story with a real \
     detail in it, one thing they gave you, and a last line to the room. Aim for 500 to 750 \
     words - three to five minutes out loud - and read it from paper rather than memory. \
     Say the funny thing if it was funny, because a laugh at a funeral is a relief and not \
     a mistake, and put the plainest, truest sentence you have near the end where it will \
     be heard."
        .to_string()
}

// --- following up a job application ------------------------------------------------------------

static FOLLOW_UP_THANKS: LazyLock<String> = LazyLock::new(|| string("FOLLOW_UP_THANKS"));
static FOLLOW_UP_CHASE: LazyLock<String> = LazyLock::new(|| string("FOLLOW_UP_CHASE"));
static FOLLOW_UP_AFTER_INTERVIEW: LazyLock<String> = LazyLock::new(|| string("FOLLOW_UP_AFTER_INTERVIEW"));
static FOLLOW_UP_WHEN: LazyLock<String> = LazyLock::new(|| string("FOLLOW_UP_WHEN"));
static INTERVIEW_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("INTERVIEW_WORDS"));
static CHASE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("CHASE_WORDS"));
static WHEN_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("WHEN_WORDS"));

/// `re.search(r"\b" + re.escape(word) + r"\b", said)`: the word stands on its own in the line.
pub fn _said(said: &str, word: &str) -> bool {
    sh::re_search(&format!(r"\b{}\b", re::escape(word)), said, 0)
}

/// A line that asks how long to wait before following up, with nothing named in it at all.
pub fn _follow_up_general(_memory: &Memory, _understood: &Understood) -> String {
    FOLLOW_UP_WHEN.clone()
}

/// The thank-you note if there was an interview, the chase if there has not been one yet, and
/// the wait first when the line asks how long.
pub fn _follow_up(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::or(&understood.norm_text, &understood.text);
    let thanked = INTERVIEW_WORDS.iter().any(|w| _said(said, w));
    let chasing = CHASE_WORDS.iter().any(|w| _said(said, w));
    if WHEN_WORDS.iter().any(|w| _said(said, w)) {
        return if thanked { FOLLOW_UP_AFTER_INTERVIEW.clone() } else { FOLLOW_UP_CHASE.clone() };
    }
    if thanked {
        return format!("{} {}", *FOLLOW_UP_THANKS, *FOLLOW_UP_CHASE);
    }
    if chasing {
        return format!("{} {}", *FOLLOW_UP_CHASE, *FOLLOW_UP_THANKS);
    }
    format!("{} {}", *FOLLOW_UP_THANKS, *FOLLOW_UP_CHASE)
}

// --- time off -------------------------------------------------------------------------------------

static EARLY_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("EARLY_WORDS"));
static YES_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("YES_WORDS"));
static TIME_OFF_SCRIPT: LazyLock<String> = LazyLock::new(|| string("TIME_OFF_SCRIPT"));
static EARLY_SCRIPT: LazyLock<String> = LazyLock::new(|| string("EARLY_SCRIPT"));
static TIME_OFF_SCRIPT_YES: LazyLock<String> = LazyLock::new(|| string("TIME_OFF_SCRIPT_YES"));
static EARLY_SCRIPT_YES: LazyLock<String> = LazyLock::new(|| string("EARLY_SCRIPT_YES"));
static ALREADY_HAD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ALREADY_HAD"));
static HAS_THE_TIME: LazyLock<Vec<String>> = LazyLock::new(|| strs("_HAS_THE_TIME"));
static AWAY_OR_HOME: LazyLock<Vec<String>> = LazyLock::new(|| strs("_AWAY_OR_HOME"));
static USE_THE_WEEK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_USE_THE_WEEK"));
static ASKING_FRAME: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ASKING_FRAME"));
static MONTHS_NAMED: LazyLock<Vec<String>> = LazyLock::new(|| strs("_MONTHS_NAMED"));
static HOW_LONG: LazyLock<Vec<String>> = LazyLock::new(|| strs("_HOW_LONG"));
static ASKS_TO_HAVE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ASKS_TO_HAVE"));

/// " line " in lower case, as the time-off rows read it.
fn spaced_lower(understood: &Understood) -> String {
    format!(" {} ", sh::clean_or_text(understood).to_lowercase())
}

/// True when the line is a request for time off, not a statement that the time is theirs.
pub fn _asks_for_the_time(understood: &Understood) -> bool {
    let said = spaced_lower(understood);
    if sh::any_in(&said, &ALREADY_HAD) {
        return false;
    }
    if sh::any_in(&said, &AWAY_OR_HOME) {
        return false;
    }
    if sh::any_in(&said, &USE_THE_WEEK) {
        return false;
    }
    if (sh::any_in(&said, &MONTHS_NAMED) || sh::any_in(&said, &HOW_LONG)) && !sh::any_in(&said, &ASKS_TO_HAVE) {
        return false;
    }
    if sh::any_in(&said, &HAS_THE_TIME) && !sh::any_in(&said, &ASKING_FRAME) {
        return false;
    }
    sh::any_in(&said, &ASKING_FRAME)
}

/// A day off wants the dates; an afternoon off wants the time and who covers it.
pub fn _time_off(_memory: &Memory, understood: &Understood) -> String {
    let said = spaced_lower(understood);
    let early = EARLY_WORDS.iter().any(|p| said.contains(&format!(" {p} ")) || said.contains(p.as_str()));
    let asking = sh::any_in(&said, &YES_WORDS);
    if early {
        return if asking { EARLY_SCRIPT_YES.clone() } else { EARLY_SCRIPT.clone() };
    }
    if asking {
        TIME_OFF_SCRIPT_YES.clone()
    } else {
        TIME_OFF_SCRIPT.clone()
    }
}

// --- nerves, said in the person's own words ------------------------------------------------------

static SIGNS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.wording.SIGNS")
        .items()
        .iter()
        .filter_map(|row| {
            let row = row.items();
            Some((row.first()?.strs(), row.get(1)?.as_str()?.to_string()))
        })
        .collect()
});
static SIGN_PLURAL: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.wording._SIGN_PLURAL"));

/// The sign of nerves the person actually named, in their words, or None when they named none.
pub fn sign_said(text: &str) -> Option<String> {
    let said = text.to_lowercase();
    SIGNS.iter().find(|(phrases, _)| sh::any_in(&said, phrases)).map(|(_, label)| label.clone())
}

/// `Shaky hands before an interview are ...` in the person's own words ("That feeling ... is"
/// when they named no sign).
pub fn nerves_opening(text: &str, situation: &str, tail: &str) -> String {
    match sign_said(text) {
        None => format!("That feeling {situation} is {tail}"),
        Some(label) => {
            let verb = if SIGN_PLURAL.contains(&label) { " are " } else { " is " };
            format!("{label} {situation}{verb}{tail}")
        }
    }
}

// --- comfort --------------------------------------------------------------------------------------

static COMFORT: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.wording.COMFORT")
        .items()
        .iter()
        .filter_map(|row| {
            let row = row.items();
            Some((row.first()?.strs(), row.get(1)?.as_str()?.to_string()))
        })
        .collect()
});
static COMFORT_SITUATIONS: LazyLock<Vec<String>> = LazyLock::new(|| strs("COMFORT_SITUATIONS"));
static COMFORT_UNNAMED: LazyLock<Vec<String>> = LazyLock::new(|| strs("COMFORT_UNNAMED"));
static COMFORT_LAST: LazyLock<String> = LazyLock::new(|| string("COMFORT_LAST"));

/// `re.search(r"\b" + re.escape(word) + r"\b", text)`.
fn word_in(word: &str, text: &str) -> bool {
    sh::re_search(&format!(r"\b{}\b", re::escape(word)), text, 0)
}

/// The first sentence for the bad thing the line names, and the same rule under it; None when
/// nothing in the line says what happened.
pub fn _comfort(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", understood.clean.to_lowercase());
    // LEANS_ON: the line this one follows (`leans_on["words"]`) is a dict only on a later turn;
    // the typed reading holds no words for it, so `earlier` is "" here.
    let earlier = String::new();
    let both = format!("{said}{earlier}");
    for (words, reply) in COMFORT.iter() {
        for word in words {
            if word.contains(' ') {
                let w = format!(" {word} ");
                if said.contains(&w) || earlier.contains(&w) {
                    return Some(reply.clone());
                }
            } else if word_in(word, &both) {
                return Some(reply.clone());
            }
        }
    }
    if !COMFORT_SITUATIONS.iter().chain(COMFORT_UNNAMED.iter()).any(|w| word_in(w, &both)) {
        return None;
    }
    Some(COMFORT_LAST.clone())
}

// --- notices --------------------------------------------------------------------------------------

static BEING_WRITTEN: LazyLock<&'static Re> = LazyLock::new(|| rx("_BEING_WRITTEN"));

/// Whether the notice is being written now, rather than reported after the fact.
pub fn _the_notice_is_being_written(understood: &Understood) -> bool {
    BEING_WRITTEN.is_match(&understood.clean)
}

// --- entry lambdas --------------------------------------------------------------------------------

/// card-something's guard: a line that gives an age is the birthday card's, not this one's.
pub fn lambda_1299(reading: &Understood) -> bool {
    !_names_an_age(sh::clean_or_text(reading))
}

/// The cancelling-friend row's guard: not when the person is the one who did it (the repair).
pub fn lambda_2329(reading: &Understood) -> bool {
    !dup::owns_the_deed(reading)
}

// --- registry wrappers ----------------------------------------------------------------------------

fn g_lambda_1299(_f: &FnRef, u: &Understood) -> bool {
    lambda_1299(u)
}
fn g_lambda_2329(_f: &FnRef, u: &Understood) -> bool {
    lambda_2329(u)
}
fn g_asks_for_the_time(_f: &FnRef, u: &Understood) -> bool {
    _asks_for_the_time(u)
}
fn g_not_a_hurt_body(_f: &FnRef, u: &Understood) -> bool {
    _not_a_hurt_body(u)
}
fn g_the_notice_is_being_written(_f: &FnRef, u: &Understood) -> bool {
    _the_notice_is_being_written(u)
}
fn g_screens_are_a_childs(_f: &FnRef, u: &Understood) -> bool {
    screens_are_a_childs(u)
}
fn g_screens_are_somebody_elses(_f: &FnRef, u: &Understood) -> bool {
    screens_are_somebody_elses(u)
}

fn r_card_birthday(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_card_birthday(m, u))
}
fn r_card_plain(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_card_plain(m, u))
}
fn r_comfort(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _comfort(m, u)
}
fn r_eulogy(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_eulogy(m, u))
}
fn r_follow_up(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_follow_up(m, u))
}
fn r_raise_problem(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_raise_problem(m, u))
}
fn r_saying_it(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _saying_it(m, u)
}
fn r_spelling(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _spelling(m, u)
}
fn r_time_off(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_time_off(m, u))
}
fn r_thanks_note(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(thanks_note(m, u))
}

fn s_asks_for_the_words(_f: &FnRef, line: &str) -> bool {
    _asks_for_the_words(line)
}
fn s_names_an_age(_f: &FnRef, line: &str) -> bool {
    _names_an_age(line)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.wording:<lambda>:1299", g_lambda_1299),
    ("bot.knowledge.wording:<lambda>:2329", g_lambda_2329),
    ("bot.knowledge.wording:_asks_for_the_time:910", g_asks_for_the_time),
    ("bot.knowledge.wording:_not_a_hurt_body:465", g_not_a_hurt_body),
    ("bot.knowledge.wording:_the_notice_is_being_written:1183", g_the_notice_is_being_written),
    ("bot.knowledge.wording:screens_are_a_childs:55", g_screens_are_a_childs),
    ("bot.knowledge.wording:screens_are_somebody_elses:65", g_screens_are_somebody_elses),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.wording:_card_birthday:158", r_card_birthday),
    ("bot.knowledge.wording:_card_plain:198", r_card_plain),
    ("bot.knowledge.wording:_comfort:1143", r_comfort),
    ("bot.knowledge.wording:_eulogy:710", r_eulogy),
    ("bot.knowledge.wording:_follow_up:947", r_follow_up),
    ("bot.knowledge.wording:_raise_problem:471", r_raise_problem),
    ("bot.knowledge.wording:_saying_it:492", r_saying_it),
    ("bot.knowledge.wording:_spelling:662", r_spelling),
    ("bot.knowledge.wording:_time_off:937", r_time_off),
    ("bot.knowledge.wording:thanks_note:582", r_thanks_note),
];

pub const SHAPES: &[(&str, ShapeFn)] = &[
    ("bot.knowledge.wording:_asks_for_the_words:232", s_asks_for_the_words),
    ("bot.knowledge.wording:_names_an_age:134", s_names_an_age),
];
