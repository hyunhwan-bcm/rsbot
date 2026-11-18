//! Port of PyBot's `bot/knowledge/fitness.py`: getting started, how much to do, and what a
//! short session is worth (the replies that read the minutes the person has, and the guards
//! that keep a fare, an hour or somebody else's ache off the fitness rows).

use crate::data::{self, FnRef};
use crate::knowledge::body;
use crate::knowledge::d1_dup_g4 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Module globals from the bundle, by their Python name.
macro_rules! global {
    ($name:ident: re) => {
        static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.knowledge.fitness.", stringify!($name))));
    };
    ($name:ident: strs) => {
        static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.knowledge.fitness.", stringify!($name))));
    };
    ($name:ident: string) => {
        static $name: LazyLock<String> =
            LazyLock::new(|| data::string(concat!("bot.knowledge.fitness.", stringify!($name))));
    };
}

global!(_MINUTES_NAMED: re);
global!(_PRESS_UP_PLAN: string);
global!(_PULL_UP_PLAN: string);
global!(_ACHE_DAY: re);
global!(_ACHE_SOMEONE_ELSE: re);
global!(THE_COUNT_OF_SESSIONS: strs);
global!(THE_HOUR_ITSELF: strs);
global!(_MONEY_IN_THE_LINE: re);
global!(_A_TIME_IS_ASKED: strs);
global!(_ABS_NAMES: strs);
global!(_EVERY_DAY_ASK: re);
global!(_PACE_WORD: re);
global!(_NOT_A_WALK: re);

/// `_MINUTE_WORDS`: a number said as a word, by the word.
static _MINUTE_WORDS: LazyLock<HashMap<String, i64>> = LazyLock::new(|| {
    data::get("bot.knowledge.fitness._MINUTE_WORDS")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0)))
        .collect()
});

/// `_MINUTES_COUNTED`: the word people say for a number of minutes, by the number.
static _MINUTES_COUNTED: LazyLock<HashMap<i64, String>> = LazyLock::new(|| {
    data::get("bot.knowledge.fitness._MINUTES_COUNTED")
        .pairs()
        .iter()
        .map(|(k, v)| (k.as_int().unwrap_or(0), v.str().to_string()))
        .collect()
});

/// How many minutes the line is about (the largest it names), or 0 when it does not say.
pub fn _minutes(text_in: &str) -> i64 {
    static HALF_AN_HOUR: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhalf an hour\b", 0));
    static AN_HOUR: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:an|one|1)\s+hour\b", 0));
    let text = text_in.to_lowercase();
    let mut found: Vec<i64> = Vec::new();
    for m in _MINUTES_NAMED.finditer(&text) {
        let digits = m.str(1);
        let word = m.str(2).replace('-', " ");
        if !digits.is_empty() {
            // `int(digits)`: the capture is one to three digits
            if let Some(n) = dup::py_int(digits) {
                found.push(n);
            }
        } else if let Some(n) = _MINUTE_WORDS.get(&word) {
            found.push(*n);
        }
    }
    if HALF_AN_HOUR.is_match(&text) {
        found.push(30);
    } else if AN_HOUR.is_match(&text) {
        found.push(60);
    }
    found.into_iter().max().unwrap_or(0)
}

/// The number as a word where it is a word people say, else the digits.
pub fn _minute_phrase(minutes: i64) -> String {
    _MINUTES_COUNTED.get(&minutes).filter(|w| !w.is_empty()).cloned().unwrap_or_else(|| minutes.to_string())
}

/// The press-up progression, or the pull-up one when the line names a pull or chin up.
pub fn _the_rep_plan(_memory: &Memory, understood: &Understood) -> String {
    static PULL_UP: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:pull|chin)[\s-]?ups?\b", 0));
    let joined = understood.words.join(" ");
    let text = sh::or(sh::clean_or_text(understood), &joined).to_lowercase();
    if PULL_UP.is_match(&text) {
        return _PULL_UP_PLAN.clone();
    }
    _PRESS_UP_PLAN.clone()
}

/// True when the ache is the person's own and the line puts it a day or two after (DOMS).
pub fn _the_ache_has_its_day(understood: &Understood) -> bool {
    if !body::_the_part_is_a_persons(understood) {
        return false;
    }
    let said = sh::text_or_clean(understood).to_lowercase();
    if _ACHE_SOMEONE_ELSE.is_match(&said) {
        return false;
    }
    _ACHE_DAY.is_match(&said)
}

/// A circuit for the time the person says they have (five minutes still counts).
pub fn _short_workout(_memory: &Memory, understood: &Understood) -> String {
    let joined = understood.words.join(" ");
    let minutes = _minutes(sh::or(&understood.text, &joined));
    let circuit = "40 seconds each of squats, press-ups (on your knees is fine), a plank and lunges, \
                   twice round, with a minute's rest";
    let keeps = "Bodyweight work like that keeps strength and gets your heart going without a gym.";
    if minutes != 0 && minutes <= 5 {
        return format!(
            "Even {} minutes counts: 40 seconds each of squats, press-ups (on your knees is fine) \
             and a plank, twice round, and that is a real session. {keeps}",
            _minute_phrase(minutes)
        );
    }
    if minutes != 0 {
        if minutes >= 30 {
            return format!(
                "With {} minutes, do a circuit: 40 seconds each of squats, press-ups (on your knees \
                 is fine), a plank and lunges, three times round, with a minute's rest, then a brisk \
                 walk to finish. {keeps}",
                _minute_phrase(minutes)
            );
        }
        if minutes == 20 {
            return format!("With 20 minutes, do a circuit: {circuit}. {keeps}");
        }
        return format!("With {} minutes, do a circuit: {circuit}. {keeps}", _minute_phrase(minutes));
    }
    format!("In the time you have, do a circuit: {circuit}. {keeps}")
}

/// False when the line is about what something costs: the circuit is no answer to a fare.
pub fn _not_a_fare(understood: &Understood) -> bool {
    let said = sh::padded(&understood.clean.to_lowercase());
    !_MONEY_IN_THE_LINE.is_match(&said)
}

/// False when the line asks how many times a week and names no hour of the day.
pub fn _a_count_beside_the_hour(understood: &Understood) -> bool {
    let said = sh::padded(&understood.clean.to_lowercase());
    if !sh::any_in(&said, &THE_COUNT_OF_SESSIONS) {
        return true;
    }
    sh::any_in(&said, &THE_HOUR_ITSELF)
}

/// False when the line does not ask when: a session already had, or a beginner's tips.
pub fn _a_time_is_asked(understood: &Understood) -> bool {
    let said = sh::padded(&understood.clean.to_lowercase());
    sh::any_in(&said, &_A_TIME_IS_ASKED)
}

/// Short cardio for the minutes the person has, building to twenty most days.
pub fn _short_cardio(_memory: &Memory, understood: &Understood) -> String {
    let joined = understood.words.join(" ");
    let minutes = _minutes(sh::or(&understood.text, &joined));
    if minutes != 0 && minutes < 20 {
        return format!(
            "{} minutes of brisk walking, stairs or cycling still counts, and it is doing it most \
             days that builds the habit: start with the time you have and add five minutes a week \
             until you are at about twenty.",
            text::capitalize(&_minute_phrase(minutes))
        );
    }
    if minutes != 0 && minutes >= 20 {
        return format!(
            "{} minutes of brisk walking, cycling or stairs most days is plenty to build a base. \
             Add one or two short sessions where you're a bit out of breath, and you get most of \
             the benefit of longer workouts.",
            text::capitalize(&_minute_phrase(minutes))
        );
    }
    "Twenty minutes of brisk walking, cycling or stairs most days is plenty to build a base. Add \
     one or two short sessions where you're a bit out of breath, and you get most of the benefit \
     of longer workouts."
        .into()
}

/// Stretch after, not before (the opening fits whether the line asks "before" or "after").
pub fn _stretch_when(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::padded(&understood.clean.to_lowercase());
    let mut opening = "Yes, after:";
    if said.contains(" before ") && !said.contains(" after ") {
        opening = "Not before, no - after:";
    } else if said.contains(" before ") || !said.contains(" after ") {
        opening = "After, not before:";
    }
    format!(
        "{opening} stretching pays off when the muscle is warm, and holding a cold one first does \
         not prevent injury - it takes a little power off the session instead. What belongs \
         before is the moving version: leg swings, lunges, skips, five minutes of easy jogging, \
         which is what actually warms you up. The flexibility it keeps - reaching, bending, \
         getting up off the floor - comes from half a minute a side, twice, a few times a week."
    )
}

/// What abs are (when the line names them without asking how), else what toning takes.
pub fn _tone_or_abs(_memory: &Memory, understood: &Understood) -> String {
    static ASKS_HOW: LazyLock<Re> =
        LazyLock::new(|| Re::new(r"\b(?:how|get|gain|build|start|fast|quick|want|can i|should i)\b", 0));
    let text = sh::clean_or_text(understood).to_lowercase();
    let names_the_muscle = sh::any_in(&text, &_ABS_NAMES)
        || understood.words.iter().any(|w| {
            let w = w.to_lowercase();
            w == "abs" || w == "abdominals"
        });
    if names_the_muscle && !ASKS_HOW.is_match(&text) {
        return "Abs are the muscle across the front of your middle, and everyone has them - what \
                shows is the layer of fat in front, so training makes them bigger and only a \
                steady calorie deficit makes them visible. Sit-ups alone will not do it: two \
                20-minute sessions a week of squats, press ups (from the knees is fine) and a row \
                with a bag or band, plus walking, is the whole of what works."
            .into();
    }
    "Toning is building a little muscle and losing a little fat, so it needs both strength \
     work and a steady calorie deficit - there is no exercise that spot-reduces one area. \
     Two 20-minute sessions a week of squats, press ups (from the knees is fine) and a row \
     with a bag or band, plus walking, does the job without a gym."
        .into()
}

/// True when the line asks something about training every day (or carries a question mark).
pub fn _an_ask_about_training_every_day(understood: &Understood) -> bool {
    if _EVERY_DAY_ASK.is_match(&understood.clean) {
        return true;
    }
    understood.signals.question_mark
}

/// True when the line asks about a walking pace, not a walk-in centre's wait or a walk done.
pub fn _a_question_about_pace(understood: &Understood) -> bool {
    static A_WALK_DONE: LazyLock<Re> =
        LazyLock::new(|| Re::new(r"\b(i|we|he|she|they)\b[^.]*\b(walked|went|did)\b", 0));
    let clean = &understood.clean;
    if _NOT_A_WALK.is_match(clean) {
        return false;
    }
    if A_WALK_DONE.is_match(clean) {
        return false;
    }
    _PACE_WORD.is_match(clean)
}

/// The best-time-to-train row's guard (`<lambda>:696`): a count of sessions beside an hour,
/// and a time is asked.
pub fn lambda_696(understood: &Understood) -> bool {
    _a_count_beside_the_hour(understood) && _a_time_is_asked(understood)
}

// ---------------------------------------------------------------------------------------------
// Registry tables.

fn g_lambda_696(_f: &FnRef, u: &Understood) -> bool { lambda_696(u) }
fn g_a_question_about_pace(_f: &FnRef, u: &Understood) -> bool { _a_question_about_pace(u) }
fn g_an_ask_about_training_every_day(_f: &FnRef, u: &Understood) -> bool { _an_ask_about_training_every_day(u) }
fn g_not_a_fare(_f: &FnRef, u: &Understood) -> bool { _not_a_fare(u) }
fn g_the_ache_has_its_day(_f: &FnRef, u: &Understood) -> bool { _the_ache_has_its_day(u) }

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.fitness:<lambda>:696", g_lambda_696),
    ("bot.knowledge.fitness:_a_question_about_pace:1618", g_a_question_about_pace),
    ("bot.knowledge.fitness:_an_ask_about_training_every_day:257", g_an_ask_about_training_every_day),
    ("bot.knowledge.fitness:_not_a_fare:146", g_not_a_fare),
    ("bot.knowledge.fitness:_the_ache_has_its_day:88", g_the_ache_has_its_day),
];

fn r_short_cardio(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_short_cardio(m, u)) }
fn r_short_workout(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_short_workout(m, u)) }
fn r_stretch_when(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_stretch_when(m, u)) }
fn r_the_rep_plan(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_the_rep_plan(m, u)) }
fn r_tone_or_abs(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_tone_or_abs(m, u)) }

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.fitness:_short_cardio:187", r_short_cardio),
    ("bot.knowledge.fitness:_short_workout:102", r_short_workout),
    ("bot.knowledge.fitness:_stretch_when:204", r_stretch_when),
    ("bot.knowledge.fitness:_the_rep_plan:62", r_the_rep_plan),
    ("bot.knowledge.fitness:_tone_or_abs:233", r_tone_or_abs),
];
