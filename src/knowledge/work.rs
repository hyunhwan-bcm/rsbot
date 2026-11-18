//! Port of `bot/knowledge/work.py`: jobs, interviews, pay, hours, holiday, notice, colleagues
//! and what a job's day is like. The entries and tables are data from the bundle; this is the
//! logic their guards and replies run.

use crate::data::{self, FnRef};
use crate::knowledge::d2_shared;
use crate::knowledge::d2dup_g7::{cached_re, nerves_opening, py_percent};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

fn name(n: &str) -> String {
    format!("bot.knowledge.work.{n}")
}
fn re(n: &str) -> &'static Re {
    data::re(&name(n))
}
fn strs(n: &str) -> Vec<String> {
    data::strs(&name(n))
}
fn string(n: &str) -> String {
    data::string(&name(n))
}
fn str_pairs(n: &str) -> Vec<(String, String)> {
    data::get(&name(n)).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
}
fn get<'a>(map: &'a [(String, String)], key: &str) -> Option<&'a String> {
    map.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

static _SOMEONE_ELSE: LazyLock<&'static Re> = LazyLock::new(|| re("_SOMEONE_ELSE"));
static _CHOOSING_A_NAME: LazyLock<&'static Re> = LazyLock::new(|| re("_CHOOSING_A_NAME"));
static _WRITING_A_REFERENCE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_WRITING_A_REFERENCE"));
static _A_BODY_PART_THAT_HURTS: LazyLock<&'static Re> = LazyLock::new(|| re("_A_BODY_PART_THAT_HURTS"));
static _A_THING_THAT_HAPPENED: LazyLock<&'static Re> = LazyLock::new(|| re("_A_THING_THAT_HAPPENED"));
static CLAIMING_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("CLAIMING_WORDS"));
static _ASKING_TO_ASK: LazyLock<&'static Re> = LazyLock::new(|| re("_ASKING_TO_ASK"));
static A_NAMED_WAR: LazyLock<&'static Re> = LazyLock::new(|| re("A_NAMED_WAR"));
static _WHO_TO_GO_TO: LazyLock<&'static Re> = LazyLock::new(|| re("_WHO_TO_GO_TO"));
static _CRY_WORD: LazyLock<&'static Re> = LazyLock::new(|| re("_CRY_WORD"));
static _NOBODY_ELSE: LazyLock<&'static Re> = LazyLock::new(|| re("_NOBODY_ELSE"));
static _FIRST_PERSON: LazyLock<&'static Re> = LazyLock::new(|| re("_FIRST_PERSON"));
static _ANOTHER_PERSON: LazyLock<&'static Re> = LazyLock::new(|| re("_ANOTHER_PERSON"));
static _GOING_BACK_WRONG: LazyLock<String> = LazyLock::new(|| string("_GOING_BACK_WRONG"));
static _GOING_BACK_THEIRS: LazyLock<String> = LazyLock::new(|| string("_GOING_BACK_THEIRS"));
static _GOING_BACK_LEAVE: LazyLock<String> = LazyLock::new(|| string("_GOING_BACK_LEAVE"));
static _GOING_BACK_SICK: LazyLock<String> = LazyLock::new(|| string("_GOING_BACK_SICK"));
static _GOING_BACK_DONT_WANT: LazyLock<String> = LazyLock::new(|| string("_GOING_BACK_DONT_WANT"));
static _GOING_BACK_TIRED: LazyLock<String> = LazyLock::new(|| string("_GOING_BACK_TIRED"));
static _GOING_BACK_NERVES: LazyLock<String> = LazyLock::new(|| string("_GOING_BACK_NERVES"));
static _GOING_BACK_PREP: LazyLock<String> = LazyLock::new(|| string("_GOING_BACK_PREP"));
static VIOLENT_AT_WORK: LazyLock<Vec<String>> = LazyLock::new(|| strs("VIOLENT_AT_WORK"));
static _BULLY_REPLY: LazyLock<String> = LazyLock::new(|| string("_BULLY_REPLY"));
static A_JOB_IN_LINE: LazyLock<Vec<String>> = LazyLock::new(|| strs("A_JOB_IN_LINE"));
static A_DRESS_CODE_AND_OCCASION: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut v = strs("A_DRESS_CODE_IN_LINE");
    v.extend(strs("AN_OCCASION_IN_LINE"));
    v
});
static NERVES_BODY: LazyLock<Vec<String>> = LazyLock::new(|| strs("NERVES_BODY"));
static INTERVIEW_NERVES: LazyLock<String> = LazyLock::new(|| string("INTERVIEW_NERVES"));
static PRESENTATION_NERVES: LazyLock<String> = LazyLock::new(|| string("PRESENTATION_NERVES"));
static SHAPE_OF_THE_INTERVIEW: LazyLock<Vec<String>> = LazyLock::new(|| strs("SHAPE_OF_THE_INTERVIEW"));
static WHAT_HAPPENS_AT_AN_INTERVIEW: LazyLock<String> = LazyLock::new(|| string("WHAT_HAPPENS_AT_AN_INTERVIEW"));
static NERVES: LazyLock<Vec<String>> = LazyLock::new(|| strs("NERVES"));
static SAY_NO_FOOD: LazyLock<Vec<&'static Re>> = LazyLock::new(|| {
    strs("SAY_NO_FOOD").iter().map(|w| cached_re(&format!("\\b{}\\b", pyre::escape(w)), 0)).collect()
});
static SAY_NO_POLITELY: LazyLock<String> = LazyLock::new(|| string("SAY_NO_POLITELY"));
static _ANOTHER_PERSON_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ANOTHER_PERSON_WORDS"));
static _FIRST_DAY_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_FIRST_DAY_ASKS"));
static GOOD_NEWS: LazyLock<Vec<String>> = LazyLock::new(|| strs("GOOD_NEWS"));
static BREAK_WORK_CUES: LazyLock<Vec<String>> = LazyLock::new(|| strs("BREAK_WORK_CUES"));
static WORK_SUBJECT: LazyLock<Vec<String>> = LazyLock::new(|| strs("WORK_SUBJECT"));
static INJURY_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("INJURY_WORDS"));
static IN_LAW: LazyLock<&'static Re> = LazyLock::new(|| re("IN_LAW"));
static IN_LAW_JOB: LazyLock<Vec<String>> = LazyLock::new(|| strs("IN_LAW_JOB"));
static REVIEW_ANCHOR: LazyLock<Vec<String>> = LazyLock::new(|| strs("REVIEW_ANCHOR"));
static _BOSS: LazyLock<&'static Re> = LazyLock::new(|| re("_BOSS"));
static _BOSS_REPLY: LazyLock<String> = LazyLock::new(|| string("_BOSS_REPLY"));
static _PEER_REPLY: LazyLock<String> = LazyLock::new(|| string("_PEER_REPLY"));
static _DAYS_A_WEEK: LazyLock<&'static Re> = LazyLock::new(|| re("_DAYS_A_WEEK"));
static _SOMEONE_ELSES_WEEK: LazyLock<&'static Re> = LazyLock::new(|| re("_SOMEONE_ELSES_WEEK"));
static _DAYS_WORD: LazyLock<Vec<(String, f64)>> = LazyLock::new(|| {
    data::get(&name("_DAYS_WORD")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_f64().unwrap())).collect()
});
static _WHERE_THE_BALANCE_IS: LazyLock<String> = LazyLock::new(|| string("_WHERE_THE_BALANCE_IS"));
static _CARRY_OVER: LazyLock<&'static Re> = LazyLock::new(|| re("_CARRY_OVER"));
static _ASKS_THE_BALANCE: LazyLock<&'static Re> = LazyLock::new(|| re("_ASKS_THE_BALANCE"));
static _FIVE_DAY_TEXT: LazyLock<String> = LazyLock::new(|| string("_FIVE_DAY_TEXT"));
static _QUOTED_DAYS: LazyLock<&'static Re> = LazyLock::new(|| re("_QUOTED_DAYS"));
static _NOISE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_NOISE_WORDS"));
static _BABY_AT_HOME: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BABY_AT_HOME"));
static _QUIET_WITH_A_BABY: LazyLock<String> = LazyLock::new(|| string("_QUIET_WITH_A_BABY"));
static _PLAYING_NOISE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PLAYING_NOISE"));
static _QUIET_WITH_A_RADIO: LazyLock<String> = LazyLock::new(|| string("_QUIET_WITH_A_RADIO"));
static _QUIET_WITH_A_COLLEAGUE: LazyLock<String> = LazyLock::new(|| string("_QUIET_WITH_A_COLLEAGUE"));
static _QUIET_WITH_NOISE: LazyLock<String> = LazyLock::new(|| string("_QUIET_WITH_NOISE"));
static _QUIET_AT_WORK: LazyLock<String> = LazyLock::new(|| string("_QUIET_AT_WORK"));
static _WITH_SOMEONE_AT_WORK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_WITH_SOMEONE_AT_WORK"));
static _PEOPLE_AT_HOME: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PEOPLE_AT_HOME"));
static _TELL_AT_HOME: LazyLock<String> = LazyLock::new(|| string("_TELL_AT_HOME"));
static _TELL_AT_WORK: LazyLock<String> = LazyLock::new(|| string("_TELL_AT_WORK"));
static ROTA_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("ROTA_WORDS"));
static ROTA_CHANGE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("ROTA_CHANGE_WORDS"));
static FEELING_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("FEELING_WORDS"));
static _DAYS_OFF_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_DAYS_OFF_WORDS"));
static _DAYS_OFF_TEXT: LazyLock<String> = LazyLock::new(|| string("_DAYS_OFF_TEXT"));
static _LEAVE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_LEAVE_WORDS"));
static _TRAVEL_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_TRAVEL_WORDS"));
static _LEAVE_TEXT: LazyLock<String> = LazyLock::new(|| string("_LEAVE_TEXT"));
static PERMISSION_ASK: LazyLock<&'static Re> = LazyLock::new(|| re("PERMISSION_ASK"));
static _SHIFT_VERDICT_TEXT: LazyLock<String> = LazyLock::new(|| string("_SHIFT_VERDICT_TEXT"));
static _SHIFT_TEXT: LazyLock<String> = LazyLock::new(|| string("_SHIFT_TEXT"));
static _NOTICE_GARDEN: LazyLock<&'static Re> = LazyLock::new(|| re("_NOTICE_GARDEN"));
static _NOTICE_PAID_INSTEAD: LazyLock<&'static Re> = LazyLock::new(|| re("_NOTICE_PAID_INSTEAD"));
static _NOTICE_CAN_THEY: LazyLock<&'static Re> = LazyLock::new(|| re("_NOTICE_CAN_THEY"));
static _NOTICE_HOW_MUCH: LazyLock<&'static Re> = LazyLock::new(|| re("_NOTICE_HOW_MUCH"));
static _BURNOUT_WHAT: LazyLock<&'static Re> = LazyLock::new(|| re("_BURNOUT_WHAT"));
static _EACH_WAY_TIME: LazyLock<Vec<String>> = LazyLock::new(|| strs("_EACH_WAY_TIME"));
static _SUIT: LazyLock<&'static Re> = LazyLock::new(|| re("_SUIT"));
static _THE_WORK_NAMED: LazyLock<&'static Re> = LazyLock::new(|| re("_THE_WORK_NAMED"));
static _SCHOOL_LEAVER: LazyLock<&'static Re> = LazyLock::new(|| re("_SCHOOL_LEAVER"));
static _LOST_IN_WHAT_I_WANT: LazyLock<&'static Re> = LazyLock::new(|| re("_LOST_IN_WHAT_I_WANT"));
static _WORK_IS_THE_ADVICE: LazyLock<&'static Re> = LazyLock::new(|| re("_WORK_IS_THE_ADVICE"));
static _WEEKEND_AT_WORK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_WEEKEND_AT_WORK"));
static _A_ROOM_NAMED: LazyLock<&'static Re> = LazyLock::new(|| re("_A_ROOM_NAMED"));
static _STANDING_UP_HAPPENS: LazyLock<&'static Re> = LazyLock::new(|| re("_STANDING_UP_HAPPENS"));
static _A_BODY_UP_WORD: LazyLock<&'static Re> = LazyLock::new(|| re("_A_BODY_UP_WORD"));
static _MEET_VERB: LazyLock<&'static Re> = LazyLock::new(|| re("_MEET_VERB"));
static JOB_PLACES: LazyLock<Vec<(String, Vec<&'static Re>)>> = LazyLock::new(|| {
    data::get(&name("JOB_PLACES"))
        .items()
        .iter()
        .map(|r| {
            let words = r.items()[1].strs();
            (r.items()[0].str().to_string(), words.iter().map(|w| bounded(w)).collect())
        })
        .collect()
});
static JOB_PLACE_LANDMARK: LazyLock<&'static Re> = LazyLock::new(|| re("JOB_PLACE_LANDMARK"));
static JOB_PLACE_AS_A_CONSTRAINT: LazyLock<&'static Re> = LazyLock::new(|| re("JOB_PLACE_AS_A_CONSTRAINT"));
static JOB_PLACE_REPLIES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("JOB_PLACE_REPLIES"));
static _A_WHAT_TO_DO_ASK: LazyLock<&'static Re> = LazyLock::new(|| re("_A_WHAT_TO_DO_ASK"));
static _ONE_OFF_STALL: LazyLock<&'static Re> = LazyLock::new(|| re("_ONE_OFF_STALL"));
static _SELL_FOOD_REPLY: LazyLock<String> = LazyLock::new(|| string("_SELL_FOOD_REPLY"));
static _WORK_WORDS: LazyLock<Vec<(Vec<String>, String, Vec<String>)>> = LazyLock::new(|| {
    data::get(&name("_WORK_WORDS"))
        .items()
        .iter()
        .map(|r| (r.items()[0].strs(), r.items()[1].str().to_string(), r.items()[2].strs()))
        .collect()
});
static DAY_TABLE: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("DAY_TABLE"));
static DAY_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("DAY_WORDS"));
/// `_job_named`'s pairs: DAY_ALIASES, then each DAY_TABLE key and its plural, with their regexes.
static JOB_PAIRS: LazyLock<Vec<(String, String, &'static Re)>> = LazyLock::new(|| {
    let mut pairs: Vec<(String, String)> = data::get(&name("DAY_ALIASES"))
        .items()
        .iter()
        .map(|r| (r.items()[0].str().to_string(), r.items()[1].str().to_string()))
        .collect();
    for (key, _) in DAY_TABLE.iter() {
        pairs.push((key.clone(), key.clone()));
        pairs.push((format!("{key}s"), key.clone()));
    }
    pairs
        .into_iter()
        .map(|(word, name)| {
            let r = cached_re(&format!("\\b{}\\b", pyre::escape(&word)), 0);
            (word, name, r)
        })
        .collect()
});
static _ABOUT_SOMETHING: LazyLock<&'static Re> = LazyLock::new(|| re("_ABOUT_SOMETHING"));
static PAPER_ROUND_PAY_ASK: LazyLock<&'static Re> = LazyLock::new(|| re("PAPER_ROUND_PAY_ASK"));
static PAPER_ROUND_WORTH_ASK: LazyLock<&'static Re> = LazyLock::new(|| re("PAPER_ROUND_WORTH_ASK"));
static PAPER_ROUND_AGE_ASK: LazyLock<&'static Re> = LazyLock::new(|| re("PAPER_ROUND_AGE_ASK"));
static PAPER_ROUND_SOMEONE: LazyLock<&'static Re> = LazyLock::new(|| re("PAPER_ROUND_SOMEONE"));
static PAPER_ROUND_PAY: LazyLock<String> = LazyLock::new(|| string("PAPER_ROUND_PAY"));
static PAPER_ROUND_WORTH: LazyLock<String> = LazyLock::new(|| string("PAPER_ROUND_WORTH"));
static PAPER_ROUND_AGE: LazyLock<String> = LazyLock::new(|| string("PAPER_ROUND_AGE"));
static PAPER_ROUND_WHY: LazyLock<String> = LazyLock::new(|| string("PAPER_ROUND_WHY"));
static PAPER_ROUND_SELF: LazyLock<String> = LazyLock::new(|| string("PAPER_ROUND_SELF"));
static PAY_ALIASES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get(&name("PAY_ALIASES"))
        .items()
        .iter()
        .map(|r| (r.items()[0].str().to_string(), r.items()[1].str().to_string()))
        .collect()
});
static PAY_TABLE: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("PAY_TABLE"));
static _PAY_ASK_IS_THEIRS: LazyLock<&'static Re> = LazyLock::new(|| re("_PAY_ASK_IS_THEIRS"));

static NOT_WORD_CHARS: LazyLock<Re> = LazyLock::new(|| Re::new("[^a-z0-9' ]+", 0));
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new("[a-z']+", 0));

/// `(?<![a-z0-9])word(?![a-z0-9])`.
fn bounded(word: &str) -> &'static Re {
    cached_re(&format!("(?<![a-z0-9]){}(?![a-z0-9])", pyre::escape(word)), 0)
}

/// Guard: their own dismissal, not someone else's (unless the line says it was them).
pub fn _it_is_their_own_dismissal(understood: &Understood) -> bool {
    static MINE: LazyLock<Re> =
        LazyLock::new(|| Re::new("(?<![a-z0-9])(me|my job|my last job|my own job|mine|myself)(?![a-z0-9])", 0));
    let said = format!(" {} ", _said_text(understood).to_lowercase());
    if !_SOMEONE_ELSE.is_match(&said) {
        return true;
    }
    MINE.is_match(&said)
}

/// Guard: not choosing a name (for a business, a baby...).
pub fn _not_choosing_a_name(understood: &Understood) -> bool {
    !_CHOOSING_A_NAME.is_match(&_said_text(understood).to_lowercase())
}

/// Guard: writing a reference for someone, not needing one.
pub fn _writing_one_not_needing_one(understood: &Understood) -> bool {
    _said(understood, &_WRITING_A_REFERENCE)
}

/// Guard: a body part that hurts is a health line unless something happened at work.
pub fn _not_a_body_part_that_hurts(understood: &Understood) -> bool {
    let said = _said_text(understood).to_lowercase();
    if !_A_BODY_PART_THAT_HURTS.is_match(&said) {
        return true;
    }
    _A_THING_THAT_HAPPENED.is_match(&said)
}

/// Guard: the line is about claiming (expenses, benefits...).
pub fn _it_is_about_claiming(understood: &Understood) -> bool {
    _said(understood, &CLAIMING_WORDS)
}

/// True unless the line only asks whether it may ask.
pub fn _the_asking_has_its_own_answer(understood: &Understood) -> bool {
    !_ASKING_TO_ASK.is_match(&_said_text(understood))
}

/// Guard: not a named war ("the cold war").
pub fn _a_named_war(understood: &Understood) -> bool {
    !A_NAMED_WAR.is_match(&_said_text(understood))
}

/// The line: clean, else as typed.
pub fn _said_text(understood: &Understood) -> String {
    d2_shared::or(&understood.clean, &understood.text).to_string()
}

/// Guard: "who do i go to" is a route, which this row is not.
pub fn _a_who_ask_is_a_route(understood: &Understood) -> bool {
    !_WHO_TO_GO_TO.is_match(&_said_text(understood).to_lowercase())
}

/// True when the line says one of the phrases as whole words.
pub fn _said<S: AsRef<str>>(understood: &Understood, phrases: &[S]) -> bool {
    _said_in(&_said_text(understood), phrases)
}

/// `_said({"clean": line}, phrases)`.
pub fn _said_in<S: AsRef<str>>(line: &str, phrases: &[S]) -> bool {
    let clean = NOT_WORD_CHARS.sub(" ", &format!(" {} ", line.to_lowercase()));
    phrases.iter().any(|p| bounded(p.as_ref()).is_match(&clean))
}

/// Whose tears the line is about: "mine", "theirs", or None.
pub fn _whose_tears(text: &str) -> Option<&'static str> {
    let words = LETTERS.findall_str(&text.to_lowercase());
    for i in 0..words.len() {
        let said = words[i..(i + 2).min(words.len())].join(" ");
        if _CRY_WORD.fullmatch(&words[i]).is_none() && _CRY_WORD.fullmatch(&said).is_none() {
            continue;
        }
        let near = words[i.saturating_sub(4)..i].join(" ");
        if i > 0 && ["me", "us", "myself"].contains(&words[i - 1].as_str()) {
            return Some("mine");
        }
        if _NOBODY_ELSE.is_match(&near) {
            return Some("mine");
        }
        if _FIRST_PERSON.is_match(&near) && !_ANOTHER_PERSON.is_match(&near) {
            return Some("mine");
        }
        if _ANOTHER_PERSON.is_match(&near) {
            return Some("theirs");
        }
    }
    None
}

/// Guard: the tears are not someone else's.
pub fn _my_own_tears(understood: &Understood) -> bool {
    _whose_tears(&_said_text(understood)) != Some("theirs")
}

/// Guard: the tears are not the speaker's own.
pub fn _someone_elses_tears(understood: &Understood) -> bool {
    _whose_tears(&_said_text(understood)) != Some("mine")
}

/// Going back to work: hating it, a partner's return, after leave, after illness, not wanting
/// to, tired, nervous, or how to prepare.
pub fn _going_back(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let u = understood;
    if _said(u, &["hate", "hate it", "regret", "miserable", "cant face", "can't face"]) {
        return Some(_GOING_BACK_WRONG.clone());
    }
    if _said(u, &["my wife", "my husband", "my partner"]) {
        return Some(_GOING_BACK_THEIRS.clone());
    }
    if _said(
        u,
        &[
            "maternity", "maternity leave", "paternity", "adoption leave", "parental leave", "shared parental", "my baby",
            "the baby", "got a baby", "new baby",
        ],
    ) {
        return Some(_GOING_BACK_LEAVE.clone());
    }
    if _said(
        u,
        &[
            "off sick", "sick leave", "sick note", "signed off", "fit note", "phased return", "stress", "burnout",
            "anxiety", "depression", "depressed", "hospital", "operation", "surgery", "miscarriage", "bereavement",
            "grief", "sickness", "illness", "unwell", "after being ill", "flu", "covid", "time off",
        ],
    ) {
        return Some(_GOING_BACK_SICK.clone());
    }
    if _said(
        u,
        &[
            "dont want to go back", "don't want to go back", "do not want to go back", "dont want to return",
            "don't want to return", "not looking forward", "dreading going back", "dreading it", "cant face going back",
            "can't face going back",
        ],
    ) {
        return Some(_GOING_BACK_DONT_WANT.clone());
    }
    if _said(u, &["exhausted", "knackered", "shattered", "tired", "drained", "wrecked", "done in", "knackering"]) {
        return Some(_GOING_BACK_TIRED.clone());
    }
    if _said(
        u,
        &[
            "nervous", "anxious", "anxiety", "worried", "worry", "dread", "dreading", "scared", "guilty", "frightened",
            "terrified",
        ],
    ) {
        return Some(_GOING_BACK_NERVES.clone());
    }
    Some(_GOING_BACK_PREP.clone())
}

/// Bullying at work, with the police first when there is a threat of violence.
pub fn _bullying(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if _said(understood, &VIOLENT_AT_WORK) {
        return Some(format!(
            "If someone is threatening you or could hurt you, ring 999 if you are in danger now and tell the police \
as well as your employer: a threat of violence is a crime, not only a workplace problem. {}",
            *_BULLY_REPLY
        ));
    }
    Some(_BULLY_REPLY.clone())
}

/// Guard: a dress code at work, not for an occasion off the job.
pub fn not_a_dress_code_off_the_job(understood: &Understood) -> bool {
    if _said(understood, &A_JOB_IN_LINE) {
        return true;
    }
    !_said(understood, &A_DRESS_CODE_AND_OCCASION)
}

/// Interview nerves, with the body's signs named back when the line names them.
pub fn _interview_nerves(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if _said(understood, &NERVES_BODY) {
        return Some(format!(
            "{}. Eat something plain an hour before, sip water rather than a big coffee, and breathe out slowly for a \
minute in the queue. {}",
            nerves_opening(
                &_said_text(understood),
                "before an interview",
                "adrenaline, not an illness, and it fades in the first two minutes of talking"
            ),
            *INTERVIEW_NERVES
        ));
    }
    Some(INTERVIEW_NERVES.clone())
}

/// Nerves before a presentation, with the body's signs named back when the line names them.
pub fn _presentation_nerves(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if _said(understood, &NERVES_BODY) {
        let advice: &str = &PRESENTATION_NERVES;
        let mut chars = advice.chars();
        let first: String = chars.next().map(|c| c.to_lowercase().collect()).unwrap_or_default();
        return Some(format!(
            "{}. Eat something plain first, then {first}{}",
            nerves_opening(&_said_text(understood), "before a talk", "adrenaline, and it drops once you are a minute in"),
            chars.as_str()
        ));
    }
    Some(PRESENTATION_NERVES.clone())
}

/// Preparing for an interview: what happens at one, with nerves, or the four stories.
pub fn _interview_prep(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let reply = "Prepare four stories in situation-task-action-result shape: a week you organised, a difficult \
customer or colleague, something you fixed, and a time you got it wrong and changed. Then write three questions for \
them about the team, the first six months and what good looks like. Say the answers out loud once - saying it is \
different from thinking it.";
    if _said(understood, &SHAPE_OF_THE_INTERVIEW) {
        return Some(WHAT_HAPPENS_AT_AN_INTERVIEW.clone());
    }
    if _said(understood, &NERVES) {
        return Some(format!(
            "Nerves before an interview are normal, and they shrink once you have something ready. {reply}"
        ));
    }
    Some(reply.to_string())
}

/// Saying no: to food at a table, or politely in general.
pub fn _say_no(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = d2_shared::or(&understood.clean, &understood.text).to_lowercase();
    if SAY_NO_FOOD.iter().any(|r| r.is_match(&text)) {
        return Some(
            "Keep it to four words and take the decision out of it: \"None for me, thank you\", then ask them \
something so the attention moves off you. A reason at a table is an invitation to be talked round, and one no is \
enough."
                .to_string(),
        );
    }
    Some(SAY_NO_POLITELY.clone())
}

/// A first day at work: feeling lost, what to ask, or what to take.
pub fn _first_day(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let u = understood;
    if _said(u, &NERVES) && _said(u, &_ANOTHER_PERSON_WORDS) && !_said(u, &_FIRST_DAY_ASKS) {
        return None;
    }
    if _said(
        u,
        &[
            "feel lost", "feeling lost", "dont know anyone", "don't know anyone", "no one talks", "nobody talks",
            "on my own", "left alone", "started last week", "started this week", "im new here", "i'm new here",
            "overwhelmed",
        ],
    ) {
        return Some(
            "Write down every question you have for a week, then take the list to your manager at the end of it: \
\"These are the things I keep guessing at - can we go through them?\" Ask two people if you can sit with them for an \
hour and watch how the work is really done, and say yes to the tea round, because the small talk is where the useful \
information is. Being new is a licence to ask that you only get once, so use it in the first month rather than saving \
it."
                .to_string(),
        );
    }
    if _said(
        u,
        &[
            "ask on my first day", "ask on the first day", "ask questions", "questions to ask", "what to ask",
            "what questions", "which questions", "questions on my first day", "questions on the first day",
            "first day questions", "ask my new team", "ask my new boss", "ask my new manager",
        ],
    ) {
        let reply = "Ask four things: who does what on the team, what a good first month looks like, who checks your \
work before it goes out, and what you should read or learn first. Write the answers down as you hear them, because \
four answers said out loud on day one are gone by Friday.";
        if _said(u, &GOOD_NEWS) {
            return Some(format!("Congratulations. {reply}"));
        }
        return Some(reply.to_string());
    }
    let reply = "Take a notebook, a pen, your ID and the paperwork they asked for, and dress one notch smarter than \
you think you need to. Your only jobs on day one are names, where the kettle and toilet are, and writing down what you \
are told. Asking questions is not a weakness in week one - it is the whole job.";
    if _said(u, &GOOD_NEWS) {
        return Some(format!(
            "Congratulations - the first week is mostly names and questions, so go easy on yourself. {reply}"
        ));
    }
    Some(reply.to_string())
}

/// Pay and hours: zero hours, unpaid leave, breaks, the 48-hour week, changed hours, overtime,
/// or the contract first; None for a line that is not about work.
pub fn _pay_and_hours(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    static BREAK_WHEN: LazyLock<Re> =
        LazyLock::new(|| Re::new("\\b(?:when|if|every time)\\b[^.?!]*\\bbreak(?:s|ing)?\\b", 0));
    static BREAK_DOWN: LazyLock<Re> = LazyLock::new(|| Re::new("\\bbreak(?:s|ing)?\\s+down\\b", 0));
    let u = understood;
    let time_to_fill = |u: &Understood| {
        _said(u, &["an hour", "half an hour", "an hour or two", "to kill"])
            && _said(u, &["what can i do", "what should i do", "what to do", "shall i do", "anything to do"])
    };
    if time_to_fill(u) {
        return None;
    }
    if _said(u, &["zero hours", "zero hour contract", "casual contract", "no guaranteed"]) {
        return Some(
            "A zero-hours contract means no guaranteed hours: they offer work and you can say no, and you must be \
paid for every hour you do work. An exclusivity clause - one that stops you working elsewhere - is void, and after 12 \
weeks of regular hours you can ask for a fixed-hours contract. You still build up holiday pay on the hours you work, \
at about 12 per cent of them."
                .to_string(),
        );
    }
    if _said(u, &["unpaid leave", "unpaid time off", "leave without pay", "time off without pay"]) {
        return Some(
            "There is no general right to unpaid leave: your employer can say no, and asking is a request, not a \
notice. The exceptions are time off for dependants in an emergency, which is unpaid but protected, and parental leave. \
Ask early and in writing, say what will be covered while you are away, and offer the dates."
                .to_string(),
        );
    }
    if _said(u, &BREAK_WORK_CUES) && _said(u, &["lunch", "break", "breaks", "rest break", "rest breaks", "breather"]) {
        return Some(
            "On a shift longer than six hours you are entitled to a 20-minute break, and to 11 hours' rest between \
shifts; under six hours there is no legal minimum, and the break can be unpaid. If your lunch is being cut short every \
day, put it in writing once - the rota and the times - and ask for the 20 minutes back."
                .to_string(),
        );
    }
    if _said(u, &["48", "forty eight", "long hours", "too many hours", "maximum hours", "how many hours"]) {
        return Some(
            "The legal limit is an average 48-hour week over 17 weeks, and you can only go over it if you signed an \
opt-out in writing - which you can cancel in writing, usually with seven days' notice. If your weeks are over 48 on \
average, ask for the calculation in writing; the 48-hour week cannot be traded away by a verbal yes."
                .to_string(),
        );
    }
    if _said(
        u,
        &["change my hours", "changed my hours", "change my shift", "rota", "different hours", "new hours", "change my contract"],
    ) {
        return Some(
            "Your hours can only be changed with your agreement, unless your contract has a clause that lets them - \
and even then they have to use it reasonably, with notice. Say no in writing and ask for the reason: if it is a real \
business need they should consult you first, and a big change with no agreement can be a breach of contract. A union \
rep or ACAS will back you up for free."
                .to_string(),
        );
    }
    if _said(u, &["overtime", "extra hours", "work late", "stay late", "time and a half"]) {
        if _said(
            u,
            &[
                "make me", "make us", "can my boss", "can my employer", "can my work", "have to", "refuse", "stay late",
                "staying late", "work late", "working late", "without pay", "unpaid overtime",
            ],
        ) {
            return Some(
                "Only the hours your contract sets, and beyond those staying late is voluntary - there is no legal \
right to extra pay for extra hours unless the contract promises overtime, only that your average pay still clears the \
minimum wage. Check what it says about the overtime rate and when it starts, and put any change to your hours in \
writing so it is clear which ones were agreed."
                    .to_string(),
            );
        }
        return Some(
            "Paid overtime depends on your contract: if it says overtime is paid, they must pay it, but if it says \
nothing there is no legal right to extra pay for extra hours - only that your average pay still clears the minimum \
wage. Check the contract for the rate and when it starts, and put any change to your hours in writing so it is clear \
which ones were agreed."
                .to_string(),
        );
    }
    if time_to_fill(u) {
        return None;
    }
    if !_said(u, &WORK_SUBJECT) {
        return None;
    }
    let line = NOT_WORD_CHARS.sub(" ", &format!(" {} ", _said_text(u).to_lowercase()));
    if BREAK_WHEN.is_match(&line) {
        return None;
    }
    if BREAK_DOWN.is_match(&line) && !_said(u, &["shift", "shifts", "rota", "work", "working", "job", "boss"]) {
        return None;
    }
    if _said(u, &INJURY_WORDS) && !_said(u, &BREAK_WORK_CUES) {
        return None;
    }
    if IN_LAW.is_match(&line) && !_said(u, &IN_LAW_JOB) {
        return None;
    }
    Some(
        "Your contract is the place to look first: it sets your hours, your breaks and whether overtime is paid. On top \
of it the law gives you a 20-minute break on a shift over six hours, 11 hours between shifts, and an average 48-hour \
week unless you signed an opt-out. If something is not being followed, ask for it in writing - that is what an \
employment adviser or a union rep needs to help."
            .to_string(),
    )
}

/// True when the text names a slip a review would bring up.
pub fn _about_a_slip(text: &str) -> bool {
    _said_in(text, &REVIEW_ANCHOR)
}

/// Talking about a slip in a review without sounding defensive; None when no slip is named.
pub fn _defensive_review(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = _said_text(understood);
    let turns = &memory.turns;
    let earlier = turns[turns.len().saturating_sub(3)..].iter().map(|t| t.user.as_str()).collect::<Vec<_>>().join(" ");
    if !_about_a_slip(&line) && !_about_a_slip(&earlier) {
        return None;
    }
    Some(
        "Lead with the fact, then what you did about it, then what you learned: \"The project slipped by six weeks; I \
flagged it in May, we re-planned, and I now build in a review at the halfway point.\" Talking about the fix is what \
stops it sounding defensive, so don't explain the cause twice."
            .to_string(),
    )
}

/// A difficult boss, or a difficult peer.
pub fn _difficult_colleague(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = understood.clean.to_lowercase();
    Some(if _BOSS.is_match(&text) { _BOSS_REPLY.clone() } else { _PEER_REPLY.clone() })
}

/// 5.6 weeks of a working week of `days` days, as "22.4" or "28".
pub fn _weeks_figure(days: f64) -> String {
    let shown = format!("{:.1}", 5.6 * days);
    match shown.strip_suffix(".0") {
        Some(whole) => whole.to_string(),
        None => shown,
    }
}

/// The days a week the speaker said they work, from the last three lines.
pub fn _the_week_they_said(memory: &Memory) -> Option<String> {
    static FIRST: LazyLock<Re> = LazyLock::new(|| Re::new("\\bi\\b|\\bim\\b|\\bi'm\\b", 0));
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
        let said = turn.user.to_lowercase();
        let Some(found) = _DAYS_A_WEEK.search(&said) else { continue };
        let start = text::char_index(&said, found.start()).saturating_sub(40);
        let around = &said[text::byte_at(&said, start)..found.end()];
        if _SOMEONE_ELSES_WEEK.is_match(around) {
            continue;
        }
        if FIRST.is_match(around) {
            return Some(found.str(1).to_string());
        }
    }
    None
}

fn days_of(said: &str) -> f64 {
    match _DAYS_WORD.iter().find(|(k, _)| k == said) {
        Some((_, v)) => *v,
        None => text::strip(said).parse().expect("float()"),
    }
}

/// Where their holiday balance is, with the part-time figure for the days they work.
pub fn _their_balance(said: Option<&str>) -> String {
    let mut worked = "Part-time is the same in proportion.".to_string();
    if let Some(said) = said.filter(|s| !s.is_empty()) {
        let days = days_of(said);
        if days != 5.0 {
            worked = format!("Part-time is the same in proportion - {said} days a week is {} days.", _weeks_figure(days));
        }
    }
    format!(
        "Full-time you get 5.6 weeks of paid holiday a year, which is 28 days with the bank holidays counted inside \
it. {worked} {}",
        *_WHERE_THE_BALANCE_IS
    )
}

/// Holiday entitlement: carrying over, the balance, a part-time week, a figure quoted, or 28 days.
pub fn _holiday_entitlement(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = _said_text(understood).to_lowercase();
    if _CARRY_OVER.is_match(&line) {
        return Some(
            "Carry-over is your contract's, not the law's: the 28 days is a floor, and the 4 weeks of it can only be \
carried over where the contract or your employer's policy allows it, which is why the handbook is the place to check. \
A common rule is up to 5 days carried to the end of March; a use-it-or-lose-it deadline is lawful when you had the \
chance to take the time. If you could not take it because you were off sick or your request was refused, you \
generally keep the right to it, and ACAS is free if they will not give it."
                .to_string(),
        );
    }
    let own_line = understood.text.to_lowercase();
    let mut part_time = _DAYS_A_WEEK.search(&own_line);
    if let Some(m) = &part_time {
        if _SOMEONE_ELSES_WEEK.is_match(&own_line[..m.end()]) {
            part_time = None;
        }
    }
    let said = match &part_time {
        Some(m) => Some(m.str(1).to_string()),
        None => _the_week_they_said(memory),
    };
    if _ASKS_THE_BALANCE.is_match(&line) {
        return Some(_their_balance(said.as_deref()));
    }
    if let Some(said) = said.filter(|s| !s.is_empty()) {
        let days = days_of(&said);
        if days == 5.0 {
            return Some(_FIVE_DAY_TEXT.clone());
        }
        return Some(format!(
            "In the UK it is 5.6 weeks of paid holiday a year, which is 28 days if you work five days a week - so \
{said} days a week is {} days - and bank holidays can be counted in that. Part-time staff get the same in proportion. \
Agency, zero-hours and casual workers build it up too. Your contract can give more than that, never less, so check it; \
if holiday is refused or unpaid, ACAS is free and takes it from there.",
            _weeks_figure(days)
        ));
    }
    if let Some(quoted) = _QUOTED_DAYS.search(&line) {
        let days: i64 = quoted.str(1).parse().expect("int()");
        let verdict = if days >= 28 {
            format!("{days} days is above the 28-day minimum, so there is nothing short there")
        } else {
            format!("{days} days is below the 28 days a full-time job has to get")
        };
        return Some(format!(
            "{verdict}. In the UK it is 5.6 weeks of paid holiday a year, and bank holidays sit inside that rather than \
on top of it, so 28 days is a floor and not a target. Under 28 is only right for a part-time week - four days a week \
is 22.4 days, three days is 16.8 - so ask for your figure in writing and check it against the days you work. If it \
comes up short, ACAS is free and takes it on."
        ));
    }
    Some(_FIVE_DAY_TEXT.clone())
}

/// The smell named, said at work and at home.
pub fn _the_smell(clean: &str) -> (&'static str, &'static str) {
    static SMOKE: LazyLock<Re> = LazyLock::new(|| Re::new("\\bsmok|cigarette|cigar|vap(?:e|ing)", 0));
    static BREATH: LazyLock<Re> = LazyLock::new(|| Re::new("\\bbreath\\b|bad breath|halitosis|garlic|onion", 0));
    static FEET: LazyLock<Re> =
        LazyLock::new(|| Re::new("\\bfeet\\b|\\bfoot\\b|\\bshoes\\b|trainers|socks|\\bsweaty feet\\b", 0));
    if SMOKE.is_match(clean) {
        return ("you smelled of smoke in the meeting", "your jacket still smells of smoke");
    }
    if BREATH.is_match(clean) {
        return ("your breath across the desk was hard to miss", "your breath in the car was hard to miss");
    }
    if FEET.is_match(clean) {
        return ("your shoes under the desk were hard to miss", "your trainers by the door were hard to miss");
    }
    ("you had a smell about you in the meeting", "your shirts still smell after the wash")
}

/// Guard: a person at work is this row's only when the line names a noise.
pub fn _a_person_and_a_noise(understood: &Understood) -> bool {
    let clean = understood.clean.to_lowercase();
    if !_says_someone_at_work(&clean) {
        return true;
    }
    _NOISE_WORDS.iter().any(|w| _says_it(&clean, w))
}

/// A quiet hour: with a baby at home, a radio at work, a loud colleague, noise, or at work.
pub fn _quiet_hour(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    if _BABY_AT_HOME.iter().any(|w| _says_it(&clean, w)) {
        return Some(_QUIET_WITH_A_BABY.clone());
    }
    if _says_someone_at_work(&clean) {
        if _PLAYING_NOISE.iter().any(|w| _says_it(&clean, w)) {
            return Some(_QUIET_WITH_A_RADIO.clone());
        }
        if ["phone", "calls", "call", "talking", "talks", "talks loudly", "chatter"].iter().any(|w| _says_it(&clean, w)) {
            return Some(_QUIET_WITH_A_COLLEAGUE.clone());
        }
        return Some(_QUIET_WITH_NOISE.clone());
    }
    Some(_QUIET_AT_WORK.clone())
}

/// True when the line says the word on its own.
pub fn _says_it(clean: &str, word: &str) -> bool {
    bounded(word).is_match(clean)
}

/// True when the line names someone at work.
pub fn _says_someone_at_work(clean: &str) -> bool {
    _WITH_SOMEONE_AT_WORK.iter().any(|w| _says_it(clean, w))
}

/// How to tell someone they smell: someone at home, or at work.
pub fn _tell_about_a_smell(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    let (at_work, at_home) = _the_smell(&clean);
    if _PEOPLE_AT_HOME.iter().any(|p| bounded(p).is_match(&clean)) {
        return Some(py_percent(&_TELL_AT_HOME, &[at_home]));
    }
    Some(py_percent(&_TELL_AT_WORK, &[at_work]))
}

/// A changed shift: days off, leave, whether they can, or what to do; None for a feeling.
pub fn _shift_changed_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = understood.clean.to_lowercase();
    let said = d2_shared::or(&understood.text, &text).to_lowercase();
    let names_it = _said_in(&said, &ROTA_WORDS) || _said_in(&said, &ROTA_CHANGE_WORDS);
    if !names_it && _said_in(&said, &FEELING_WORDS) {
        return None;
    }
    if _said_in(&text, &_DAYS_OFF_WORDS) {
        return Some(_DAYS_OFF_TEXT.clone());
    }
    if _said_in(&text, &_LEAVE_WORDS) {
        if _said_in(&text, &_TRAVEL_WORDS) {
            return None;
        }
        return Some(_LEAVE_TEXT.clone());
    }
    if PERMISSION_ASK.is_match(&text) {
        return Some(_SHIFT_VERDICT_TEXT.clone());
    }
    Some(_SHIFT_TEXT.clone())
}

/// A notice period: garden leave, pay in lieu, whether they can hold you to it, or its length.
pub fn _notice_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = _said_text(understood).to_lowercase();
    let opening = if _NOTICE_GARDEN.is_match(&line) {
        "Garden leave is being paid to stay at home for your notice: your contract still runs, your pay and holiday \
carry on, and you must be ready to come back if they call. It is the employer's choice, and it cannot run past your \
notice. "
    } else if _NOTICE_PAID_INSTEAD.is_match(&line) {
        "Often yes: many contracts let your employer pay you for the notice instead of you working it, and they can \
insist on it - so check for a payment in lieu clause. "
    } else if _NOTICE_CAN_THEY.is_match(&line) && !_NOTICE_HOW_MUCH.is_match(&line) {
        "Yes, if the contract says so: a notice period binds both sides, so they can hold you to the full notice and \
you can hold them to theirs. "
    } else {
        "Your contract decides the length - usually a week to three months - so read that or the intranet, not your \
memory. "
    };
    Some(format!(
        "{opening}In the UK the statutory minimum is a week after a month's service, rising by a week a year up to \
twelve. They can put you on garden leave, and most contracts let either side agree to stop early, with pay in lieu of \
the rest. To serve less, ask early and in writing: unused holiday can usually be taken inside the notice, and a start \
date at the next job is the reason a boss says yes."
    ))
}

/// Burnout: what it is, and the smallest real break first.
pub fn _burnout_card(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = text::strip(&understood.clean).to_lowercase();
    let advice = "Take the smallest real break first: a day off this week, and one week booked in the diary that \
nobody can move. Then bring the load to your manager as a list of what is due and what you can do, and ask which to \
drop - burnout is usually a staffing problem landing on one person. Tell your GP if you are not sleeping or eating \
properly, because a sick note is a normal thing to ask for and it protects your job.";
    if _BURNOUT_WHAT.match_(&clean).is_some() {
        return Some(format!(
            "Burnout is the exhaustion that comes when the demands of a job outrun what you recover between shifts: \
tired after a normal night, flat or cynical about the work, and sure you are getting worse at it. It is a work problem \
rather than a weakness, and it eases when the load changes. {advice}"
        ));
    }
    Some(advice.to_string())
}

/// Guard: "each way" is a commute only when it is said as time.
pub fn _a_journey_that_costs_time(understood: &Understood) -> bool {
    if !_said(understood, &["each way"]) {
        return true;
    }
    _said(understood, &_EACH_WAY_TIME)
}

/// The line squashed and lower-cased (clean, else text).
fn squashed(understood: &Understood) -> String {
    text::squash(&_said_text(understood).to_lowercase())
}

/// True unless the line is about a suit to wear (and not suiting a kind of work).
pub fn _not_a_thing_to_wear(understood: &Understood) -> bool {
    let line = squashed(understood);
    if !_SUIT.is_match(&line) {
        return true;
    }
    _THE_WORK_NAMED.is_match(&line)
}

/// True unless the line is the school leaver's question.
pub fn _not_the_school_leaver_question(understood: &Understood) -> bool {
    !_SCHOOL_LEAVER.is_match(&squashed(understood))
}

/// Guard: not lost in what they want, not something to wear, not the school leaver's question.
pub fn _not_saying_they_are_lost_or_wearing_it(understood: &Understood) -> bool {
    _not_lost_in_what_they_want(understood)
        && _not_a_thing_to_wear(understood)
        && _not_the_school_leaver_question(understood)
}

/// True unless the line says they do not know what they want.
pub fn _not_lost_in_what_they_want(understood: &Understood) -> bool {
    !_LOST_IN_WHAT_I_WANT.is_match(&squashed(understood))
}

/// Guard: the weekend is a working one (and work is not the advice asked for).
pub fn _the_weekend_is_a_working_one(understood: &Understood) -> bool {
    if _WORK_IS_THE_ADVICE.is_match(&_said_text(understood).to_lowercase()) {
        return false;
    }
    _said(understood, &_WEEKEND_AT_WORK)
}

/// True unless the line is a body standing up ("stand up from the desk").
pub fn a_body_standing(understood: &Understood) -> bool {
    static STAND_UP: LazyLock<Re> = LazyLock::new(|| Re::new("\\bstand(?:s|ing)?\\s+up\\b|\\bgetting up\\b", pyre::I));
    let mut said = d2_shared::or(&understood.text, &understood.clean).to_string();
    if said.is_empty() {
        said = understood.words.join(" ");
    }
    if said.is_empty() || _A_ROOM_NAMED.is_match(&said) {
        return true;
    }
    if _STANDING_UP_HAPPENS.is_match(&said) {
        return false;
    }
    if _A_BODY_UP_WORD.is_match(&said) && STAND_UP.is_match(&said) {
        return false;
    }
    true
}

/// Guard: a meeting room, not the verb "meet".
pub fn _the_room_and_not_the_verb_meet(understood: &Understood) -> bool {
    if !a_body_standing(understood) {
        return false;
    }
    let mut said = d2_shared::or(&understood.text, &understood.clean).to_string();
    if said.is_empty() {
        said = understood.words.join(" ");
    }
    if !_MEET_VERB.is_match(&said) {
        return true;
    }
    _A_ROOM_NAMED.is_match(&said)
}

/// A job in a named kind of place (a care home, a supermarket...), or None.
pub fn _job_in_a_place(_memory: Option<&Memory>, understood: &Understood) -> Option<String> {
    let clean = NOT_WORD_CHARS.sub(" ", &format!(" {} ", _said_text(understood).to_lowercase()));
    for (place, words) in JOB_PLACES.iter() {
        for word in words {
            if let Some(m) = word.search(&clean) {
                if !JOB_PLACE_LANDMARK.is_match(&clean[..m.start()]) && !JOB_PLACE_AS_A_CONSTRAINT.is_match(&clean) {
                    return Some(get(&JOB_PLACE_REPLIES, place).expect("KeyError").clone());
                }
            }
        }
    }
    None
}

/// Guard: the line names no kind of place a job could be in.
pub fn _not_a_named_place(understood: &Understood) -> bool {
    _job_in_a_place(None, understood).is_none()
}

/// True when the line asks what to do.
pub fn _asks_what_to_do(clean: &str) -> bool {
    _A_WHAT_TO_DO_ASK.is_match(clean)
}

/// Selling food from home: a one-off stall, or registering with the council.
pub fn _sell_food_from_home(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = understood.clean.to_lowercase();
    if _ONE_OFF_STALL.is_match(&text) {
        return Some(
            "A one-off stall - a school fair, a charity bake sale, a fete - is not a food business, so nothing has to \
be registered: keep it clean, keep it cold, label the allergens and you are done. Selling regularly from home is the \
free council kitchen register at least 28 days ahead, with a Level 2 hygiene certificate and public liability \
insurance after that."
                .to_string(),
        );
    }
    Some(_SELL_FOOD_REPLY.clone())
}

/// A work word explained (redundancy, appraisal, BTEC...), unless the line asks something else.
pub fn _work_word(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = &understood.clean;
    let words = &understood.words;
    let said = |key: &String| text.contains(key.as_str()) || words.contains(key);
    for (keys, reply, avoid) in _WORK_WORDS.iter() {
        if !keys.iter().any(said) {
            continue;
        }
        if avoid.iter().any(said) {
            continue;
        }
        return Some(reply.clone());
    }
    None
}

/// The job the line names (the longest name wins) and where it starts (a character index),
/// or (None, -1).
pub fn _job_named(line: &str) -> (Option<String>, isize) {
    let mut best: Option<(usize, &String, usize)> = None;
    for (word, name, r) in JOB_PAIRS.iter() {
        if name.is_empty() {
            continue;
        }
        if let Some(hit) = r.search(line) {
            let n = word.chars().count();
            if best.is_none_or(|b| n > b.0) {
                best = Some((n, name, hit.start()));
            }
        }
    }
    match best {
        Some((_, name, at)) => (Some(name.clone()), text::char_index(line, at) as isize),
        None => (None, -1),
    }
}

/// What a day in the job the line names is like, or None.
pub fn _job_day(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    static HOW_DO_I: LazyLock<Re> =
        LazyLock::new(|| Re::new("\\b(?:what|how) (?:do|does|should|can|shall|could) (?:i|we)\\b", 0));
    let line = format!(" {}", _said_text(understood).to_lowercase());
    if _ABOUT_SOMETHING.is_match(&line) {
        return None;
    }
    let (name, at) = _job_named(&line);
    let name = name.filter(|n| !n.is_empty())?;
    if HOW_DO_I.is_match(&line) {
        return None;
    }
    let opens: Vec<isize> = DAY_WORDS.iter().filter(|w| line.contains(w.as_str())).map(|w| text::find(&line, w)).collect();
    if opens.is_empty() || *opens.iter().min().unwrap() > at {
        return None;
    }
    get(&DAY_TABLE, &name).cloned()
}

/// A paper round: the pay, whether it is worth it, the age, for someone else, or for themselves.
pub fn _paper_round(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = _said_text(understood).to_lowercase();
    if PAPER_ROUND_PAY_ASK.is_match(&said) {
        return Some(PAPER_ROUND_PAY.clone());
    }
    if PAPER_ROUND_WORTH_ASK.is_match(&said) {
        return Some(PAPER_ROUND_WORTH.clone());
    }
    if PAPER_ROUND_AGE_ASK.is_match(&said) {
        return Some(PAPER_ROUND_AGE.clone());
    }
    if PAPER_ROUND_SOMEONE.is_match(&said) {
        return Some(PAPER_ROUND_WHY.clone());
    }
    Some(PAPER_ROUND_SELF.clone())
}

/// A criminal record and work: "" for a DBS wait, when a conviction is spent, or what to declare.
pub fn _a_criminal_record(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    static DBS_WAIT: LazyLock<Re> = LazyLock::new(|| {
        Re::new("\\b(how long|how many weeks|how many days|takes?|come back|wait|waiting|delay|delayed|chase)\\b", 0)
    });
    static SPENT: LazyLock<Re> = LazyLock::new(|| {
        Re::new(
            "\\b(how long|how many years|when does it|expire|expires|spent after|come off|comes off|goes away|gone away|stay on|stays on|wiped|cleared)\\b",
            0,
        )
    });
    static ON_RECORD: LazyLock<Re> = LazyLock::new(|| Re::new("\\bgo(?:es)? on (?:my|your) record\\b", 0));
    let said = d2_shared::or(&reading.norm_text, &reading.clean).to_lowercase();
    if said.contains("dbs") && DBS_WAIT.is_match(&said) {
        return Some(String::new());
    }
    if SPENT.is_match(&said) || ON_RECORD.is_match(&said) {
        return Some(
            "Most convictions become spent after a set time that runs from the end of the sentence - a year for a fine \
or a community order, and longer for prison - and after that you can answer no to most employers. A prison sentence of \
over four years never becomes spent, and a caution is spent as soon as it is given."
                .to_string(),
        );
    }
    Some(
        "A record is not an automatic no: most employers can ask only about unspent convictions, and a spent one or a \
caution need not be declared unless the job is exempt - work with children, patients, or in the justice system. Those \
jobs ask for a standard or enhanced DBS check, where an old minor caution can be filtered out, so apply and see rather \
than deciding for them."
            .to_string(),
    )
}

/// The job the line names for the pay table: a pay alias first, else the day table's name.
pub fn _pay_job_named(line: &str) -> Option<String> {
    let mut best: Option<(usize, &String)> = None;
    for (alias, name) in PAY_ALIASES.iter() {
        if line.contains(&format!(" {alias} ")) {
            let n = alias.chars().count();
            if best.is_none_or(|b| n > b.0) {
                best = Some((n, name));
            }
        }
    }
    if let Some((_, name)) = best {
        return Some(name.clone());
    }
    _job_named(line).0
}

/// What the job the line names pays, unless the line asks about someone else's pay.
pub fn _job_pay(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = format!(" {} ", _said_text(understood).to_lowercase());
    if _PAY_ASK_IS_THEIRS.is_match(&line) {
        return None;
    }
    let name = _pay_job_named(&line)?;
    get(&PAY_TABLE, &name).cloned()
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.work:_a_journey_that_costs_time:1411", |_: &FnRef, u: &Understood| _a_journey_that_costs_time(u)),
    ("bot.knowledge.work:_a_named_war:279", |_: &FnRef, u: &Understood| _a_named_war(u)),
    ("bot.knowledge.work:_a_person_and_a_noise:1120", |_: &FnRef, u: &Understood| _a_person_and_a_noise(u)),
    ("bot.knowledge.work:_a_who_ask_is_a_route:301", |_: &FnRef, u: &Understood| _a_who_ask_is_a_route(u)),
    ("bot.knowledge.work:_it_is_about_claiming:246", |_: &FnRef, u: &Understood| _it_is_about_claiming(u)),
    ("bot.knowledge.work:_it_is_their_own_dismissal:178", |_: &FnRef, u: &Understood| _it_is_their_own_dismissal(u)),
    ("bot.knowledge.work:_my_own_tears:362", |_: &FnRef, u: &Understood| _my_own_tears(u)),
    ("bot.knowledge.work:_not_a_body_part_that_hurts:233", |_: &FnRef, u: &Understood| _not_a_body_part_that_hurts(u)),
    ("bot.knowledge.work:_not_a_named_place:1681", |_: &FnRef, u: &Understood| _not_a_named_place(u)),
    ("bot.knowledge.work:_not_choosing_a_name:197", |_: &FnRef, u: &Understood| _not_choosing_a_name(u)),
    ("bot.knowledge.work:_not_saying_they_are_lost_or_wearing_it:1461", |_: &FnRef, u: &Understood| {
        _not_saying_they_are_lost_or_wearing_it(u)
    }),
    ("bot.knowledge.work:_someone_elses_tears:367", |_: &FnRef, u: &Understood| _someone_elses_tears(u)),
    ("bot.knowledge.work:_the_room_and_not_the_verb_meet:1555", |_: &FnRef, u: &Understood| {
        _the_room_and_not_the_verb_meet(u)
    }),
    ("bot.knowledge.work:_the_weekend_is_a_working_one:1493", |_: &FnRef, u: &Understood| {
        _the_weekend_is_a_working_one(u)
    }),
    ("bot.knowledge.work:_writing_one_not_needing_one:220", |_: &FnRef, u: &Understood| _writing_one_not_needing_one(u)),
    ("bot.knowledge.work:not_a_dress_code_off_the_job:543", |_: &FnRef, u: &Understood| not_a_dress_code_off_the_job(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.work:_a_criminal_record:8802", |_: &FnRef, m: &mut Memory, u: &Understood| _a_criminal_record(m, u)),
    ("bot.knowledge.work:_bullying:514", |_: &FnRef, m: &mut Memory, u: &Understood| _bullying(m, u)),
    ("bot.knowledge.work:_burnout_card:1379", |_: &FnRef, m: &mut Memory, u: &Understood| _burnout_card(m, u)),
    ("bot.knowledge.work:_defensive_review:816", |_: &FnRef, m: &mut Memory, u: &Understood| _defensive_review(m, u)),
    ("bot.knowledge.work:_difficult_colleague:853", |_: &FnRef, m: &mut Memory, u: &Understood| _difficult_colleague(m, u)),
    ("bot.knowledge.work:_first_day:663", |_: &FnRef, m: &mut Memory, u: &Understood| _first_day(m, u)),
    ("bot.knowledge.work:_going_back:483", |_: &FnRef, m: &mut Memory, u: &Understood| _going_back(m, u)),
    ("bot.knowledge.work:_holiday_entitlement:950", |_: &FnRef, m: &mut Memory, u: &Understood| _holiday_entitlement(m, u)),
    ("bot.knowledge.work:_interview_nerves:556", |_: &FnRef, m: &mut Memory, u: &Understood| _interview_nerves(m, u)),
    ("bot.knowledge.work:_interview_prep:610", |_: &FnRef, m: &mut Memory, u: &Understood| _interview_prep(m, u)),
    ("bot.knowledge.work:_job_day:8658", |_: &FnRef, m: &mut Memory, u: &Understood| _job_day(m, u)),
    ("bot.knowledge.work:_job_in_a_place:1668", |_: &FnRef, m: &mut Memory, u: &Understood| _job_in_a_place(Some(&*m), u)),
    ("bot.knowledge.work:_job_pay:9120", |_: &FnRef, m: &mut Memory, u: &Understood| _job_pay(m, u)),
    ("bot.knowledge.work:_notice_reply:1347", |_: &FnRef, m: &mut Memory, u: &Understood| _notice_reply(m, u)),
    ("bot.knowledge.work:_paper_round:8776", |_: &FnRef, m: &mut Memory, u: &Understood| _paper_round(m, u)),
    ("bot.knowledge.work:_pay_and_hours:706", |_: &FnRef, m: &mut Memory, u: &Understood| _pay_and_hours(m, u)),
    ("bot.knowledge.work:_presentation_nerves:589", |_: &FnRef, m: &mut Memory, u: &Understood| _presentation_nerves(m, u)),
    ("bot.knowledge.work:_quiet_hour:1137", |_: &FnRef, m: &mut Memory, u: &Understood| _quiet_hour(m, u)),
    ("bot.knowledge.work:_say_no:633", |_: &FnRef, m: &mut Memory, u: &Understood| _say_no(m, u)),
    ("bot.knowledge.work:_sell_food_from_home:7052", |_: &FnRef, m: &mut Memory, u: &Understood| _sell_food_from_home(m, u)),
    ("bot.knowledge.work:_shift_changed_reply:1266", |_: &FnRef, m: &mut Memory, u: &Understood| _shift_changed_reply(m, u)),
    ("bot.knowledge.work:_tell_about_a_smell:1163", |_: &FnRef, m: &mut Memory, u: &Understood| _tell_about_a_smell(m, u)),
    ("bot.knowledge.work:_work_word:8361", |_: &FnRef, m: &mut Memory, u: &Understood| _work_word(m, u)),
];
