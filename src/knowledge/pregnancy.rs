//! Port of PyBot's `bot/knowledge/pregnancy.py`: food, due dates, labour, the hospital bag and
//! birth words.

use crate::data::{self, FnRef};
use crate::knowledge::d1_dup_g7 as dup;
use crate::knowledge::d1_shared::{self as sh, ShapeFn};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

fn key(name: &str) -> String {
    format!("bot.knowledge.pregnancy.{name}")
}
fn rx(name: &str) -> &'static Re {
    data::re(&key(name))
}
static PREG_WORRY_FOODS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get(&key("PREG_WORRY_FOODS")).items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect()
});
static PREG_ANY_FOOD: LazyLock<String> = LazyLock::new(|| data::string(&key("PREG_ANY_FOOD")));
static _FEEDING_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&key("_FEEDING_WORDS")));
static _DUE_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| rx("_DUE_SHAPE"));
static _PREG_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("_PREG_WORD"));
static _PREG_SOFT: LazyLock<&'static Re> = LazyLock::new(|| rx("_PREG_SOFT"));
static _OTHER_SUBJECT: LazyLock<&'static Re> = LazyLock::new(|| rx("_OTHER_SUBJECT"));
static _DUE_ELSEWHERE: LazyLock<&'static Re> = LazyLock::new(|| rx("_DUE_ELSEWHERE"));
static _A_MILESTONE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_MILESTONE"));
static _PERIOD_ONLY: LazyLock<&'static Re> = LazyLock::new(|| rx("_PERIOD_ONLY"));
static _PERIOD_WITH_A_PREGNANCY: LazyLock<&'static Re> = LazyLock::new(|| rx("_PERIOD_WITH_A_PREGNANCY"));
static _MONTHS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    data::get(&key("_MONTHS")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0))).collect()
});
static _DAY_THEN_MONTH: LazyLock<&'static Re> = LazyLock::new(|| rx("_DAY_THEN_MONTH"));
static _MONTH_THEN_DAY: LazyLock<&'static Re> = LazyLock::new(|| rx("_MONTH_THEN_DAY"));
static _WEEKS_NOW: LazyLock<&'static Re> = LazyLock::new(|| rx("_WEEKS_NOW"));
static _SHORT_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("_SHORT_ASK"));
static _ASKED: LazyLock<&'static Re> = LazyLock::new(|| rx("_ASKED"));
static _GIVES_THE_DAY: LazyLock<&'static Re> = LazyLock::new(|| rx("_GIVES_THE_DAY"));
static _A_MONTH: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_MONTH"));
static CARD: LazyLock<String> = LazyLock::new(|| data::string(&key("CARD")));
static _TIME_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("_TIME_ASK"));
static _BIRTH_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("_BIRTH_WORD"));
static _NOT_THE_BIRTH: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&key("_NOT_THE_BIRTH")));
static _SECOND_BIRTH: LazyLock<&'static Re> = LazyLock::new(|| rx("_SECOND_BIRTH"));
static _AFTER_THE_BIRTH: LazyLock<&'static Re> = LazyLock::new(|| rx("_AFTER_THE_BIRTH"));
static _PREG_SAID: LazyLock<&'static Re> = LazyLock::new(|| rx("_PREG_SAID"));
static _PREG_DENIED: LazyLock<&'static Re> = LazyLock::new(|| rx("_PREG_DENIED"));
/// `_BIRTH_WORD_NAMES`: (word, `\bword s?\b`, reply), longest word first.
static _BIRTH_WORD_NAMES: LazyLock<Vec<(String, Re, String)>> = LazyLock::new(|| {
    data::get(&key("_BIRTH_WORD_NAMES"))
        .items()
        .iter()
        .map(|r| {
            let word = r.items()[0].str().to_string();
            let re = Re::new(&format!(r"\b{}s?\b", sh::escape(&word)), 0);
            (word, re, r.items()[1].str().to_string())
        })
        .collect()
});
static _AMBIGUOUS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&key("_AMBIGUOUS")));
static _NOT_A_BIRTH_LINE: LazyLock<&'static Re> = LazyLock::new(|| rx("_NOT_A_BIRTH_LINE"));
static _A_BIRTH_LINE: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_BIRTH_LINE"));
static _WHEN_TO_CALL: LazyLock<&'static Re> = LazyLock::new(|| rx("_WHEN_TO_CALL"));
static _DO_THEY_HURT: LazyLock<&'static Re> = LazyLock::new(|| rx("_DO_THEY_HURT"));
static _HOW_FAR_APART: LazyLock<String> = LazyLock::new(|| data::string(&key("_HOW_FAR_APART")));
static _THEY_HURT: LazyLock<String> = LazyLock::new(|| data::string(&key("_THEY_HURT")));

static RE_43: LazyLock<Re> = LazyLock::new(|| Re::new(r#"\b(?:periods?|cycles?|pregnan\w*|tests?)\b"#, 2)); // _a_late_that_is_not_a_period re.search
static RE_652: LazyLock<Re> = LazyLock::new(|| Re::new(r#"[a-z']+"#, 0)); // _asks_a_birth_time re.findall
static RE_661: LazyLock<Re> = LazyLock::new(|| Re::new(r#"\bpack\b|\bpacked\b|\bpacking\b"#, 2)); // _asks_about_the_bag re.search
static RE_663: LazyLock<Re> = LazyLock::new(|| Re::new(r#"\bhospital\b|\bmaternity\b|\bbag for\b|\bdelivery\b"#, 2)); // _asks_about_the_bag re.search
static RE_675: LazyLock<Re> = LazyLock::new(|| Re::new(r#"\bin hospital\b|\bstay in\b|\bstay at\b|\bcome home\b|\bgoing home\b|\bhow long do i stay\b"#, 2)); // _preg_labour_length re.search
static RE_688: LazyLock<Re> = LazyLock::new(|| Re::new(r#"\blongest\b|\btoo long\b|\bstall(?:ed|ing)?\b|\bhow long can (?:a |the |my )?(?:labour|labor|it|this) (?:go|last|be)\b|\bgo on for\b"#, 2)); // _preg_labour_length re.search
static RE_697: LazyLock<Re> = LazyLock::new(|| Re::new(r#"\b(?:first|second|third|1st|2nd|3rd) stage\b|\bpushing\b|\bplacenta (?:take|takes|come|comes|coming|out)\b|\bdelivery of the placenta\b|\bskin to skin\b"#, 2)); // _preg_labour_length re.search
static RE_681: LazyLock<Re> = LazyLock::new(|| Re::new(r#"\bcontractions?\b"#, 2)); // _preg_labour_length re.search
static RE_700: LazyLock<Re> = LazyLock::new(|| Re::new(r#"\bfirst\b|\b1st\b"#, 2)); // _preg_labour_length re.search
static RE_706: LazyLock<Re> = LazyLock::new(|| Re::new(r#"\bthird\b|\b3rd\b|\bplacenta\b|\bskin to skin\b"#, 2)); // _preg_labour_length re.search
static RE_716: LazyLock<Re> = LazyLock::new(|| Re::new(r#"\bwaters?\b"#, 2)); // _preg_labour_length re.search
static RE_716_49: LazyLock<Re> = LazyLock::new(|| Re::new(r#"\bbreak|broke|broken|gone|went|went off\b"#, 2)); // _preg_labour_length re.search
static RE_681_59: LazyLock<Re> = LazyLock::new(|| Re::new(r#"\blabour\b|\blabor\b|\bthe birth\b|\bgiv(?:e|ing) birth\b"#, 2)); // _preg_labour_length re.search
static RE_745: LazyLock<Re> = LazyLock::new(|| Re::new(r#"\bwhat (?:should|do) i (?:pack|take|bring)\b|\bwhat to (?:pack|take|bring)\b|\bwhat goes in\b"#, 2)); // _preg_bag re.search

#[rustfmt::skip]
const S_553: &str = " weeks to go, which puts the due date at about "; // _preg_due_date
#[rustfmt::skip]
const S_554: &str = ". That is the 40-week count, but the dating scan at 11 to 14 weeks is the date the midwife goes by, so go by what is on the notes if the scan moved it."; // _preg_due_date
#[rustfmt::skip]
const S_560: &str = "Counted from the first day of the last period, "; // _preg_due_date
#[rustfmt::skip]
const S_567: &str = "If that is not the day you meant, tell me the first day of the last period as a date and I will work it out again."; // _preg_due_date
#[rustfmt::skip]
const S_548: &str = " weeks you are past the 40-week due date, so the date on your notes and your midwife are the ones to go by now. Most places offer an induction by 41 or 42 weeks, and she will talk you through what happens next - worth asking her at your next appointment rather than waiting."; // _preg_due_date
#[rustfmt::skip]
const S_564: &str = " weeks now. The dating scan at 11 to 14 weeks is the date the midwife goes by, so if it moved the date, the scan is the one that counts."; // _preg_due_date
#[rustfmt::skip]
const S_589: &str = "Fine, and there is no evidence hair dye harms a baby: the amount that reaches your bloodstream through the scalp is tiny. Plenty of women wait until after 12 weeks before colouring, which is a reasonable choice rather than a rule. Have the patch test as usual, keep the dye off a cut or an eczema patch, and make sure the room is ventilated if you are doing it at home."; // _preg_hair_dye
#[rustfmt::skip]
const S_583: &str = "A colour is fine while you are breastfeeding: the dye does not get into your milk, so nothing reaches the baby through a feed. Keep the room aired, wear the gloves that come with the kit and do the 48-hour patch test first, because skin can react differently after a baby. Highlights or a semi-permanent keep the dye off your scalp if you would rather be careful, and a salon can do the mixing for you."; // _preg_hair_dye
#[rustfmt::skip]
const S_730: &str = "A first labour usually takes 8 to 18 hours from the first regular contractions to the birth, and a second or later baby is usually quicker. Early labour at home can run much longer than that - a day or more of stop-start contractions is normal - so it is the pattern that matters: contractions getting stronger, longer and closer together, or your waters breaking, is the point to ring the midwife."; // _preg_labour_length
#[rustfmt::skip]
const S_677: &str = "After a straightforward birth most women are home within a day: a few hours if everything is well and it is not your first, overnight or a day or two with a first baby. A caesarean is two to four days, and any concern about you or the baby adds a day, so plan for the longer end when you arrange a lift home."; // _preg_labour_length
#[rustfmt::skip]
const S_683: &str = "Each contraction usually lasts 30 to 60 seconds early on, then 60 to 90 seconds once labour is established, with a few minutes between them. They get longer, stronger and closer together as it goes on - that pattern is what tells you it is real labour rather than the practice tightenings. Ring the midwife if one lasts much longer than 90 seconds, or there is no gap between them."; // _preg_labour_length
#[rustfmt::skip]
const S_691: &str = "There is no fixed limit: a first labour is usually under 18 hours once it is established, but slow is not the same as dangerous, and they go by how it is progressing, how you are and how the baby is rather than the clock. If it stalls they offer to break the waters or a hormone drip, and if it is not moving at all they talk to you about a caesarean. Ask what has changed each time they examine you."; // _preg_labour_length
#[rustfmt::skip]
const S_711: &str = "The second stage is the pushing: about an hour on average for a first baby, sometimes two, and often 10 or 20 minutes with a second or later one. The urge to push is hard to resist and that is normal - go with it, take the breaks between contractions, and expect the midwife to guide you to slow down at the end so you tear less."; // _preg_labour_length
#[rustfmt::skip]
const S_718: &str = "Once your waters break, ring the midwife and go in to be checked - she will want to know the time, the colour and the smell. If labour has not started, most units offer to induce it within about 24 hours, because the risk of infection to you and the baby starts to rise after that. Do not use tampons or have sex once the waters have gone, and ring straight away if the water is green, brown or smelly, or if you feel unwell or the baby is not moving as usual."; // _preg_labour_length
#[rustfmt::skip]
const S_726: &str = "A second or later labour is usually quicker than the first - often 5 to 12 hours from regular contractions to the birth, and some are much faster than that. Because it can move quickly, ring the midwife early rather than timing it at home, and go in when the contractions are strong and regular."; // _preg_labour_length
#[rustfmt::skip]
const S_701: &str = "The first stage is the long part: the cervix opening to 10cm, which takes most of the 8 to 18 hours of a first labour and is much quicker with a second. It starts with contractions you can talk through and ends with strong ones a few minutes apart - that middle stretch, 4 to 6cm, is usually the hardest, so ask for pain relief before you need it."; // _preg_labour_length
#[rustfmt::skip]
const S_707: &str = "The third stage is the placenta: usually 5 to 15 minutes, and up to half an hour if you wait for it to come away by itself rather than having the injection that speeds it up. They check it is all out, and it is also when any tear is stitched, so it can feel longer than it is."; // _preg_labour_length
#[rustfmt::skip]
const S_740: &str = "Pack it by about 36 weeks - roughly a month before the due date - so it is ready if you go early. Two bags: yours with your maternity notes, a nightdress or two, big pads, toiletries, a phone charger and snacks; the baby's with nappies, vests, sleepsuits, a hat and a blanket. Keep it by the door, and add the notes last because you need them at appointments before then."; // _preg_bag
#[rustfmt::skip]
const S_747: &str = "For you: maternity notes, a nightdress or two, big pads, toiletries, a phone charger and snacks. For the baby: nappies, vests, sleepsuits, a hat and a blanket. Pack it by about 36 weeks so it is ready if you go early, keep it by the door, and leave the notes on top because you need them at appointments before then."; // _preg_bag
#[rustfmt::skip]
const S_1587: &str = "A contraction is the womb muscle tightening and then relaxing, which is what opens the cervix: it builds, holds for 45 to 60 seconds and fades, and the first ones feel like a bad period pain. They get longer, stronger and closer together as labour goes on. Time them from the start of one to the start of the next, and call the midwife when they are about five minutes apart."; // _a_labour_contraction

/// False when "late" in the line is about time (late for work), or a shift is late without a
/// period, cycle, pregnancy or test named.
pub fn _a_late_that_is_not_a_period(understood: &Understood) -> bool {
    if understood.sense.get("late").map(String::as_str) == Some("time") {
        return false;
    }
    if understood.shift.as_deref().unwrap_or("").is_empty() {
        return true;
    }
    RE_43.is_match(&understood.clean)
}

/// The card for the food the line worries about in pregnancy (the short list when none).
pub fn _preg_food(_memory: &Memory, understood: &Understood) -> String {
    let words: HashSet<&str> = understood.words.iter().map(String::as_str).collect();
    let joined = understood.words.join(" ");
    for (names, reply) in PREG_WORRY_FOODS.iter() {
        for name in names {
            if name.contains(' ') {
                if joined.contains(name.as_str()) {
                    return reply.clone();
                }
                continue;
            }
            if words.contains(name.as_str()) || name.strip_suffix('s').is_some_and(|stem| words.contains(stem)) {
                return reply.clone();
            }
        }
    }
    PREG_ANY_FOOD.clone()
}

/// A shape: the line names a month.
pub fn _gives_a_month(line: &str) -> bool {
    _A_MONTH.is_match(line)
}

/// Whether the line asks when the baby is due (a pregnancy named, not another due date).
pub fn _asks_when_due(understood: &Understood) -> bool {
    let line = text::squash(&understood.clean);
    if line.is_empty() || !_DUE_SHAPE.is_match(&line) {
        return false;
    }
    if !_ASKED.is_match(&line)
        && !(_GIVES_THE_DAY.is_match(&line) && _A_MONTH.is_match(&line))
        && !(_SHORT_ASK.is_match(&line) && text::split_ref(&line).len() <= 4)
    {
        return false;
    }
    if _DUE_ELSEWHERE.is_match(&line) || _A_MILESTONE.is_match(&line) {
        return false;
    }
    if _PERIOD_ONLY.is_match(&line) && !_PERIOD_WITH_A_PREGNANCY.is_match(&line) {
        return false;
    }
    if _PREG_WORD.is_match(&line) {
        return true;
    }
    _PREG_SOFT.is_match(&line) && !_OTHER_SUBJECT.is_match(&line)
}

/// "17 November", with the year when it is not this year's.
pub fn _the_day(date: i64, today: i64) -> String {
    let (y, m, d) = dup::civil_from_days(date);
    let said = format!("{d} {}", dup::month_name(m));
    if y == dup::civil_from_days(today).0 { said } else { format!("{said} {y}") }
}

/// The first date the line names (day then month, else month then day), within the last year.
pub fn _the_date_named(line: &str, today: i64) -> Option<i64> {
    let month_of = |name: &str| _MONTHS.iter().find(|(k, _)| k == &name.to_lowercase()).map(|(_, v)| *v);
    for m in _DAY_THEN_MONTH.finditer(line) {
        let day: i64 = m.str(1).parse().unwrap_or(0);
        if let Some(month) = month_of(m.str(2)) {
            if let Some(date) = _the_date_in_this_year(today, month, day) {
                return Some(date);
            }
        }
    }
    for m in _MONTH_THEN_DAY.finditer(line) {
        let day: i64 = m.str(2).parse().unwrap_or(0);
        if let Some(month) = month_of(m.str(1)) {
            if let Some(date) = _the_date_in_this_year(today, month, day) {
                return Some(date);
            }
        }
    }
    None
}

/// The day and month this year (or last year, when this year's is more than three days ahead),
/// as a day count; None when it is no date.
pub fn _the_date_in_this_year(today: i64, month: i64, day: i64) -> Option<i64> {
    let year = dup::civil_from_days(today).0;
    for year in [year, year - 1] {
        if !dup::valid_date(year, month, day) {
            return None;
        }
        let date = dup::days_from_civil(year, month, day);
        if date <= today + 3 {
            return Some(date);
        }
    }
    None
}

/// How many weeks pregnant the line says the person is (1 to 45).
pub fn _the_weeks_named(line: &str) -> Option<i64> {
    for m in _WEEKS_NOW.finditer(line) {
        let figure = sh::or(m.str(1), m.str(2));
        if !figure.is_empty() {
            let weeks: i64 = figure.parse().unwrap_or(0);
            if (1..=45).contains(&weeks) {
                return Some(weeks);
            }
        }
    }
    None
}

/// The due date from the weeks named or the first day of the last period (the general card when
/// the line gives neither).
pub fn _preg_due_date(_memory: &Memory, understood: &Understood) -> String {
    let line = text::squash(&understood.clean);
    let (y, m, d) = dup::TODAY;
    let today = dup::days_from_civil(y, m, d);
    if let Some(weeks) = _the_weeks_named(&line) {
        let left = 40 - weeks;
        if left < 0 {
            return format!("At {weeks}{S_548}");
        }
        let date = today + left * 7;
        return format!("Week {weeks} of 40 leaves {left}{S_553}{}{S_554}", _the_day(date, today));
    }
    if let Some(named) = _the_date_named(&line, today) {
        let date = named + 280;
        let said = format!("{S_560}{}, the due date is about {} - 40 weeks, or 280 days, later. ", _the_day(named, today), _the_day(date, today));
        let days = today - named;
        if (0..=300).contains(&days) {
            return format!("{said}That makes you {}{S_564}", days.div_euclid(7));
        }
        return format!("{said}{S_567}");
    }
    CARD.clone()
}

/// Hair dye in pregnancy, or while breastfeeding.
pub fn _preg_hair_dye(_memory: &Memory, understood: &Understood) -> String {
    let line = format!(" {} ", understood.clean);
    if _FEEDING_WORDS.iter().any(|w| line.contains(&format!(" {w} ")) || line.contains(w.as_str())) {
        return S_583.into();
    }
    S_589.into()
}

/// The clean line with whitespace collapsed.
pub fn _the_line(understood: &Understood) -> String {
    text::squash(&understood.clean)
}

/// Whether the line asks how long the birth (labour) takes, not a wait afterwards or another
/// "delivery".
pub fn _asks_a_birth_time(understood: &Understood) -> bool {
    let line = _the_line(understood);
    if line.is_empty() || !_TIME_ASK.is_match(&line) || !_BIRTH_WORD.is_match(&line) {
        return false;
    }
    if _AFTER_THE_BIRTH.is_match(&line) {
        return false;
    }
    let low = line.to_lowercase();
    !RE_652.finditer(&low).iter().any(|w| _NOT_THE_BIRTH.contains(w.as_str()))
}

/// Whether the line asks when to pack the hospital bag.
pub fn _asks_about_the_bag(understood: &Understood) -> bool {
    let line = _the_line(understood);
    if line.is_empty() || !_TIME_ASK.is_match(&line) {
        return false;
    }
    if !RE_661.is_match(&line) {
        return false;
    }
    RE_663.is_match(&line)
}

/// How long labour (or the part asked about) takes.
pub fn _preg_labour_length(_memory: &Memory, understood: &Understood) -> String {
    let line = _the_line(understood);
    if RE_675.is_match(&line) {
        return S_677.into();
    }
    if RE_681.is_match(&line) && !RE_681_59.is_match(&line) {
        return S_683.into();
    }
    if RE_688.is_match(&line) {
        return S_691.into();
    }
    if RE_697.is_match(&line) {
        if RE_700.is_match(&line) {
            return S_701.into();
        }
        if RE_706.is_match(&line) {
            return S_707.into();
        }
        return S_711.into();
    }
    if RE_716.is_match(&line) && RE_716_49.is_match(&line) {
        return S_718.into();
    }
    if _SECOND_BIRTH.is_match(&line) {
        return S_726.into();
    }
    S_730.into()
}

/// When to pack the hospital bag, or what goes in it.
pub fn _preg_bag(_memory: &Memory, understood: &Understood) -> String {
    let line = _the_line(understood);
    if RE_745.is_match(&line) {
        return S_747.into();
    }
    S_740.into()
}

/// Whether the person says they are pregnant (and does not deny it).
pub fn _said_pregnant(understood: &Understood) -> bool {
    let said = format!("{} {} {}", understood.clean, understood.text, understood.norm_text).to_lowercase();
    if _PREG_DENIED.is_match(&said) {
        return false;
    }
    _PREG_SAID.is_match(&said)
}

/// What a birth word means (the longest one named; an ambiguous word only on a birth line).
pub fn _a_birth_word(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", text::squash(&sh::or(&understood.plain, &understood.text).to_lowercase()));
    if _NOT_A_BIRTH_LINE.is_match(&said) {
        return None;
    }
    for (word, re, reply) in _BIRTH_WORD_NAMES.iter() {
        if !re.is_match(&said) {
            continue;
        }
        if _AMBIGUOUS.contains(word) && !_A_BIRTH_LINE.is_match(&said) {
            continue;
        }
        return Some(reply.clone());
    }
    None
}

/// What a contraction is, how far apart to call, or whether they hurt.
pub fn _a_labour_contraction(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = sh::or(&understood.plain, &understood.text);
    if _WHEN_TO_CALL.is_match(said) {
        return Some(_HOW_FAR_APART.clone());
    }
    if _DO_THEY_HURT.is_match(said) {
        return Some(_THEY_HURT.clone());
    }
    if !_A_BIRTH_LINE.is_match(said) {
        return None;
    }
    Some(S_1587.into())
}

macro_rules! guard {
    ($name:ident, $f:ident) => {
        fn $name(_f: &FnRef, u: &Understood) -> bool {
            $f(u)
        }
    };
}
macro_rules! reply {
    ($name:ident, $f:ident) => {
        fn $name(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
            Some($f(m, u))
        }
    };
}
guard!(g_late, _a_late_that_is_not_a_period);
guard!(g_birth_time, _asks_a_birth_time);
guard!(g_bag, _asks_about_the_bag);
guard!(g_due, _asks_when_due);
guard!(g_said, _said_pregnant);
reply!(r_bag, _preg_bag);
reply!(r_due, _preg_due_date);
reply!(r_food, _preg_food);
reply!(r_dye, _preg_hair_dye);
reply!(r_labour, _preg_labour_length);
fn r_birth_word(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _a_birth_word(m, u)
}
fn r_contraction(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _a_labour_contraction(m, u)
}
fn s_month(_f: &FnRef, line: &str) -> bool {
    _gives_a_month(line)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.pregnancy:_a_late_that_is_not_a_period:22", g_late),
    ("bot.knowledge.pregnancy:_asks_a_birth_time:645", g_birth_time),
    ("bot.knowledge.pregnancy:_asks_about_the_bag:656", g_bag),
    ("bot.knowledge.pregnancy:_asks_when_due:459", g_due),
    ("bot.knowledge.pregnancy:_said_pregnant:775", g_said),
];
pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.pregnancy:_a_birth_word:1529", r_birth_word),
    ("bot.knowledge.pregnancy:_a_labour_contraction:1577", r_contraction),
    ("bot.knowledge.pregnancy:_preg_bag:737", r_bag),
    ("bot.knowledge.pregnancy:_preg_due_date:534", r_due),
    ("bot.knowledge.pregnancy:_preg_food:192", r_food),
    ("bot.knowledge.pregnancy:_preg_hair_dye:574", r_dye),
    ("bot.knowledge.pregnancy:_preg_labour_length:667", r_labour),
];
pub const SHAPES: &[(&str, ShapeFn)] = &[("bot.knowledge.pregnancy:_gives_a_month:446", s_month)];
