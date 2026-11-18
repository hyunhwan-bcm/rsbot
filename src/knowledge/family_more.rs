//! Port of `bot/knowledge/family_more.py`: the family rows split out of `family.py` (moving
//! abroad, nappies, breastfeeding, school holidays, a police matter, a teenager's tattoo or
//! piercing). Word lists, replies and entries are data from the bundle.

use crate::data;
use crate::data::FnRef;
use crate::knowledge::d2dup_g9::{Date, _the_child_pronouns, recent, today};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{escape, Re, I};
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("bot.knowledge.family_more.{name}"))
}
fn string(name: &str) -> String {
    data::string(&format!("bot.knowledge.family_more.{name}"))
}
fn re(name: &str) -> &'static Re {
    data::re(&format!("bot.knowledge.family_more.{name}"))
}
/// A `((words...), text)` table.
fn word_rows(name: &str) -> Vec<(Vec<String>, String)> {
    data::get(&format!("bot.knowledge.family_more.{name}"))
        .items()
        .iter()
        .map(|row| (row.items()[0].strs(), row.items()[1].str().to_string()))
        .collect()
}

static _LEFT_THE_COUNTRY: LazyLock<&'static Re> = LazyLock::new(|| re("_LEFT_THE_COUNTRY"));
/// `geography.COUNTRIES` (its keys: the country names).
static COUNTRIES: LazyLock<Vec<String>> = LazyLock::new(|| {
    data::get("bot.knowledge.geography.COUNTRIES").pairs().iter().map(|(k, _)| k.str().to_string()).collect()
});
static _GROWN_UP_SAID: LazyLock<Vec<String>> = LazyLock::new(|| strs("_GROWN_UP_SAID"));
static _ADULT_AGE: LazyLock<&'static Re> = LazyLock::new(|| re("_ADULT_AGE"));
static _NAPPY_AGES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("_NAPPY_AGES"));
static _BREAST_PAIN: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BREAST_PAIN"));
static _BREAST_LENGTH: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BREAST_LENGTH"));
static _BREAST_ILL: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BREAST_ILL"));
static _BREAST_COULD_NOT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BREAST_COULD_NOT"));
static _A_NURSING_CAREER: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_NURSING_CAREER"));
static _BREAST_PUBLIC: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BREAST_PUBLIC"));
/// `_NATION_WORDS`: each nation with the patterns for its words (`\b<word>\b`).
static _NATION_WORDS: LazyLock<Vec<(String, Vec<Re>)>> = LazyLock::new(|| {
    data::get("bot.knowledge.family_more._NATION_WORDS")
        .items()
        .iter()
        .map(|row| {
            let nation = row.items()[0].str().to_string();
            let words = row.items()[1].strs().iter().map(|w| Re::new(&format!(r"\b{}\b", escape(w)), 0)).collect();
            (nation, words)
        })
        .collect()
});
static _SCHOOL_YEAR: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.family_more._SCHOOL_YEAR")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.str().to_string()))
        .collect()
});
static _THE_DATES_ARE_LOCAL: LazyLock<String> = LazyLock::new(|| string("_THE_DATES_ARE_LOCAL"));
static SUMMER_BREAK: LazyLock<String> = LazyLock::new(|| string("SUMMER_BREAK"));
static WINTER_BREAK: LazyLock<String> = LazyLock::new(|| string("WINTER_BREAK"));
static EASTER_BREAK: LazyLock<String> = LazyLock::new(|| string("EASTER_BREAK"));
static _TERM_STARTS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("_TERM_STARTS"));
static THE_FIRST_HOUR: LazyLock<String> = LazyLock::new(|| string("THE_FIRST_HOUR"));
static THE_STOPPED: LazyLock<String> = LazyLock::new(|| string("THE_STOPPED"));
static THE_QUESTIONED: LazyLock<String> = LazyLock::new(|| string("THE_QUESTIONED"));

static ASKS_THE_LENGTH: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\bhow long (?:is|are|do|does|will)\b|\bhow many weeks off\b|\bhow long (?:a|an|the)\b|\blast(?:s|ed)?\b", I)
});
static COUNTS_THE_WAIT: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:how (?:long|many)|days|weeks|till|until|countdown|left)\b", I));
static QUESTIONED: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"\b(questioned|questioning|interviewed|interview|under caution|asked questions|statement|appropriate adult)\b",
        0,
    )
});
static STOPPED: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(stopped|stop and search|searched|searches|search|handcuffed|detained|frisked)\b", 0));
static AGE_IN_THE_LINE: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\b(1[0-9]|[89])\s*(?:year|yr)s?\s*old\b|\bis\s+(1[0-9]|[89])\b|\bat\s+(1[0-9]|[89])\b|\baged\s+(1[0-9]|[89])\b", 0)
});
static EIGHTEEN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b18th\b|\b(?:is|turns|turning|she is|he is)\s+18\b|\b18\b", 0));

fn any_in<S: AsRef<str>>(text: &str, words: &[S]) -> bool {
    words.iter().any(|w| text.contains(w.as_ref()))
}

/// True when the line says the person is going to some other country, not just away.
pub fn _another_country_is_in_the_line(understood: &Understood) -> bool {
    let low = format!(" {} {} ", understood.text, understood.clean).to_lowercase();
    if _LEFT_THE_COUNTRY.is_match(&low) {
        return true;
    }
    COUNTRIES.iter().any(|name| low.contains(&format!(" {name} ")))
}

/// False when the child the line is about has a grown-up's age or life on them.
pub fn _the_child_is_not_grown(understood: &Understood) -> bool {
    let text = understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ");
    if any_in(&text, &_GROWN_UP_SAID) {
        return false;
    }
    !_ADULT_AGE.is_match(&text)
}

/// How many nappies a day, at the age the line names.
pub fn _nappies_a_day(_memory: &mut Memory, understood: &Understood) -> String {
    let said = format!(" {} ", understood.clean.to_lowercase());
    for (ages, answer) in _NAPPY_AGES.iter() {
        for age in ages {
            if said.contains(&format!(" {age} ")) {
                return answer.clone();
            }
        }
    }
    "A newborn gets through eight to ten a day - about seventy a week, so two big packs a \
     month or a subscription. It drops to around six to seven a day by six months and four \
     or five by a year. Buy one pack of newborn size and one of the next size up: they \
     grow out of newborn in a fortnight, and a size that leaks is usually too small."
        .to_string()
}

/// False when the line is about the job rather than about feeding a baby.
pub fn _not_a_nursing_career(understood: &Understood) -> bool {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let text = format!(" {} ", line.to_lowercase());
    let stripped = crate::py::text::strip(&text);
    !_A_NURSING_CAREER.iter().any(|w| text.contains(&format!(" {w} ")) || stripped.ends_with(w.as_str()))
}

/// Breastfeeding: could not, in public, pain, illness, how long, formula, or the basics.
pub fn _breastfeeding_basics(_memory: &mut Memory, understood: &Understood) -> String {
    let clean = format!(" {} ", understood.clean.to_lowercase());
    if any_in(&clean, &_BREAST_COULD_NOT) {
        return "No - a baby who is fed is fed, and how they were fed is not something you owe \
                anyone an apology for. If it did not work, that is usually a latch, a supply or a \
                medical reason rather than effort, and formula is a complete food: the health \
                visitor will help you swap over or mix the two without a fuss. If the guilt is \
                sticking, say so to your midwife or GP - it is common, it has a name, and it is \
                treatable."
            .to_string();
    }
    if any_in(&clean, &_BREAST_PUBLIC) {
        return "Yes - you can feed your baby wherever you happen to be, and in the UK the law \
                protects it: nobody can ask you to stop, move on or leave because you are \
                breastfeeding. A scarf or a layered top is plenty if you want cover, and if you \
                would rather have a quiet corner, a cafe, library or shopping centre will nearly \
                always find you one - ask the staff."
            .to_string();
    }
    if any_in(&clean, &_BREAST_PAIN) {
        return "It should not hurt beyond the first few seconds of a feed, so get help early \
                rather than pushing through: a midwife, health visitor or a local breastfeeding \
                clinic will watch a whole feed and fix the latch, which is the cause most of the \
                time. Cracked or bleeding nipples, a red painful patch on the breast with a fever, \
                or a baby who stops feeding are the reasons to be seen the same day."
            .to_string();
    }
    if any_in(&clean, &_BREAST_ILL) {
        return "Yes - keep feeding through it: your milk carries the antibodies you are making \
                against whatever it is, so it is the best protection the baby can get, and \
                stopping suddenly is what causes mastitis and a dip in supply. Drink more than \
                you feel like and feed lying down if you are wiped out, and check any medicine \
                with a pharmacist first - most are fine."
            .to_string();
    }
    if any_in(&clean, &_BREAST_LENGTH) {
        return "Only breast milk for about the first six months, then keep going alongside solid \
                food for as long as you both want - the UK advice is to two years and beyond, and \
                plenty of babies stop before that. There is no point where the milk stops being \
                worth it, and no rule you have to keep to if it is not working for you."
            .to_string();
    }
    if clean.contains(" formula") || clean.contains(" bottle ") || clean.contains(" bottles ") {
        return "Formula is a safe, complete food and a baby fed formula is a baby fed well - the \
                only thing it does not carry is the antibodies that come in your milk, which is \
                why breast milk is the default advice rather than the only good choice. Mixed \
                feeding works too, and a baby on both gets the antibodies from the feeds you do \
                give. What matters is that the baby is growing and that feeding is not miserable \
                for you."
            .to_string();
    }
    "Yes - it is the normal way to feed a baby and it is what the milk is for. Feed on \
     demand rather than by the clock: a newborn feeds eight to twelve times in 24 hours, \
     and six or more wet nappies a day with steady weight gain say it is going in. If it \
     hurts, the baby is sleepy at the breast or not putting on weight, a midwife or health \
     visitor will see you the same week."
        .to_string()
}

/// A date said the way a person says it: "Tuesday 21 July", with the year when it is not this one.
pub fn _said_date(found: &Date) -> String {
    let mut said = format!("{} {} {}", found.weekday_name(), found.day, found.month_name());
    if found.year != today().year {
        said += &format!(" {}", found.year);
    }
    said
}

/// The usual date of a school break, this year if it has not been and gone, else next year.
pub fn _school_date(month: u32, day: u32) -> Date {
    let today = today();
    for year in [today.year, today.year + 1] {
        let found = Date::new(year, month, day);
        if found >= today {
            return found;
        }
    }
    Date::new(today.year + 1, month, day)
}

/// How the date is introduced: "this year that is Wednesday 21 July", or "the next one is".
pub fn _this_or_next(found: &Date) -> String {
    if found.year == today().year {
        return format!("this year that is {}", _said_date(found));
    }
    format!("the next one is {}", _said_date(found))
}

/// "36 days (about 5 weeks)": the count in days, and in weeks when it is worth saying.
pub fn _about_days(days: i64) -> String {
    if days <= 10 {
        return format!("{days} days");
    }
    let weeks = crate::py::text::round_half_even(days as f64 / 7.0);
    format!("{days} days (about {weeks} weeks)")
}

/// "How long are the summer holidays" wants six weeks, not the days until they start.
pub fn _asks_the_length(clean: &str) -> bool {
    ASKS_THE_LENGTH.is_match(clean)
}

/// True when the line wants the days until a break rather than when it is or how long.
pub fn _counts_the_wait(clean: &str) -> bool {
    if _asks_the_length(clean) {
        return false;
    }
    COUNTS_THE_WAIT.is_match(clean)
}

/// The nation whose school year the line is asking about, or "" when it names none.
pub fn _the_nation_said(text: &str) -> String {
    for (nation, words) in _NATION_WORDS.iter() {
        if words.iter().any(|rx| rx.is_match(text)) {
            return nation.clone();
        }
    }
    String::new()
}

/// The school year of the nation the chat is in, or None for a chat that names none.
pub fn _the_nations_year(memory: &Memory, understood: &Understood) -> Option<String> {
    let text = format!(" {} ", understood.clean.to_lowercase());
    let mut nation = _the_nation_said(&text);
    // Python: `if not nation and memory` - an empty memory dict is falsy, but `recent` of it is
    // empty too, so reading the turns is the same either way.
    if nation.is_empty() {
        for turn in recent(memory, 6).iter().rev() {
            nation = _the_nation_said(&format!(" {} ", turn.user.to_lowercase()));
            if !nation.is_empty() {
                break;
            }
        }
    }
    if nation.is_empty() {
        return None;
    }
    let said = _SCHOOL_YEAR.iter().find(|(k, _)| *k == nation).map(|(_, v)| v.as_str()).unwrap_or("");
    if said.is_empty() {
        return None;
    }
    Some(format!("{said}{}", *_THE_DATES_ARE_LOCAL))
}

/// When the summer holidays start, or how long it is until they do.
pub fn _summer_break(memory: &mut Memory, understood: &Understood) -> String {
    if let Some(there) = _the_nations_year(memory, understood) {
        return there;
    }
    let clean = &understood.clean;
    let usual = _school_date(7, 21);
    if _asks_the_length(clean) {
        return format!(
            "Six weeks off: {}. The exact two days are on your school's term dates, because \
             a school can start back a day or two either side.",
            *SUMMER_BREAK
        );
    }
    if _counts_the_wait(clean) {
        let days = usual.days_since(&today());
        return format!(
            "{} until the summer holidays: {}, so they break up on {}. The exact day is on \
             your school's term dates, and it can sit a day or two either side.",
            _about_days(days),
            *SUMMER_BREAK,
            _said_date(&usual)
        );
    }
    format!(
        "{} - {}. The exact day is on your school's term dates, and it can sit a day or two \
         either side.",
        *SUMMER_BREAK,
        _this_or_next(&usual)
    )
}

/// The Christmas holidays, and the count to them when the line asks how long.
pub fn _winter_break(memory: &mut Memory, understood: &Understood) -> String {
    if let Some(there) = _the_nations_year(memory, understood) {
        return there;
    }
    let clean = &understood.clean;
    let usual = _school_date(12, 19);
    if _counts_the_wait(clean) {
        let days = usual.days_since(&today());
        return format!(
            "{} until the Christmas holidays: schools usually break up on about 19 December, \
             and {}, then go back the first Tuesday or Wednesday in January.",
            _about_days(days),
            _this_or_next(&usual)
        );
    }
    format!(
        "{}: the break-up is usually about 19 December, {}, and back on the first Tuesday or \
         Wednesday in January. Your school's term dates give the exact two days.",
        *WINTER_BREAK,
        _this_or_next(&usual)
    )
}

/// The Easter holidays, which follow Easter weekend rather than a fixed date.
pub fn _easter_break(memory: &mut Memory, understood: &Understood) -> String {
    if let Some(there) = _the_nations_year(memory, understood) {
        return there;
    }
    format!(
        "{}. Easter moves each year, so the two weeks move with it - your school's term dates \
         name the Friday it breaks up and the Monday it goes back.",
        *EASTER_BREAK
    )
}

/// Half term: a week in October, February and May, on the school's own dates.
pub fn _half_term(memory: &mut Memory, understood: &Understood) -> String {
    if let Some(there) = _the_nations_year(memory, understood) {
        return there;
    }
    "Half term is one week off: the last week of October, the middle of February and the \
     last week of May. Which week it falls in is set by your school, and the three dates \
     for the year are on its term dates page - the October one is usually the week the \
     clocks go back."
        .to_string()
}

/// The day term starts: the term the line named, or all three returns when it names none.
pub fn _back_to_school(memory: &mut Memory, understood: &Understood) -> String {
    if let Some(there) = _the_nations_year(memory, understood) {
        return there;
    }
    let clean = &understood.clean;
    for (words, said) in _TERM_STARTS.iter() {
        if any_in(clean, words) {
            return format!(
                "{said}. Your school's term dates give the exact day, and a school can add an \
                 inset day at the start of term."
            );
        }
    }
    "The summer return is the first days of September, after Christmas it is the first \
     Tuesday or Wednesday in January, and after Easter it is the Monday a week after \
     Easter Monday. Your school's term dates give the exact day, and a school can add an \
     inset day at the start of term."
        .to_string()
}

/// The shape of the school year, for a line that names no particular break.
pub fn _school_holidays_general(memory: &mut Memory, understood: &Understood) -> String {
    if let Some(there) = _the_nations_year(memory, understood) {
        return there;
    }
    let today = today();
    let summer = _school_date(7, 21);
    let winter = _school_date(12, 19);
    let soon = if winter < summer {
        format!(
            "the Christmas break, from about 19 December ({}) to the first days of January",
            _said_date(&winter)
        )
    } else {
        format!(
            "the summer break, from the third week of July ({}) to the first days of September",
            _said_date(&summer)
        )
    };
    if _counts_the_wait(&understood.clean) {
        let days = (if winter < summer { winter } else { summer }).days_since(&today);
        return format!(
            "{} until the next break - {}. The exact dates come from your school or your \
             council.",
            _about_days(days),
            soon
        );
    }
    format!(
        "The next one is {soon}. English state schools have six weeks off in summer, two at \
         Christmas, two at Easter and a week's half term in October, February and May; the \
         exact dates come from your school or your council."
    )
}

/// The first hour of an arrest, or what a stop, a search or a questioning allows.
pub fn _police_matter(_memory: &mut Memory, reading: &Understood) -> String {
    let said = if reading.norm_text.is_empty() { &reading.clean } else { &reading.norm_text }.to_lowercase();
    if QUESTIONED.is_match(&said) {
        return THE_QUESTIONED.clone();
    }
    if STOPPED.is_match(&said) {
        return THE_STOPPED.clone();
    }
    THE_FIRST_HOUR.clone()
}

/// "a 15 year old" in the line (the age), or None.
pub fn _the_age_in_the_line(understood: &Understood) -> Option<String> {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let text = if line.is_empty() { " ".to_string() } else { line.to_lowercase() };
    for m in AGE_IN_THE_LINE.finditer(&text) {
        for i in 1..=4 {
            let one = m.str(i);
            if !one.is_empty() {
                return Some(one.to_string());
            }
        }
    }
    None
}

/// The tattoo rule first, because a tattoo is what the line asked about.
pub fn _a_teenagers_tattoo(_memory: &mut Memory, understood: &Understood) -> String {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let text = if line.is_empty() { " ".to_string() } else { line.to_lowercase() };
    let age = _the_age_in_the_line(understood);
    if EIGHTEEN.is_match(&text) && age.is_none() {
        return "At 18 it is her own decision and no consent is needed, so the useful thing now is \
                which studio rather than whether: licensed only, ask to see the licence and the \
                sterile sealed needles opened in front of you, and look at healed work rather \
                than the photographs they choose. A design drawn now and lived with for a few \
                months is what saves the cover-up later. A piercing is a different rule: 16 \
                without consent for lobes and most placements, and a parent with ID below that."
            .to_string();
    }
    let who = match &age {
        Some(age) => format!(
            "In the UK a tattoo is 18 with no exceptions, so a {age} year old cannot be tattooed \
             at all - and a studio that would do one is breaking the law and is the studio to \
             avoid"
        ),
        None => "In the UK a tattoo is 18 with no exceptions, so nobody under 18 can be tattooed \
                 at all - and a studio that would do one is breaking the law and is the studio to \
                 avoid"
            .to_string(),
    };
    format!(
        "{who}. What you can say yes to now is the design: have them draw it and live with \
         it on paper for a year, because a good studio will still be there at 18. Piercings \
         are the smaller ask, and a different rule: 16 without consent for lobes and most \
         other placements, and under that a parent is usually there with ID, so ring the shop \
         first because the rules differ by council."
    )
}

/// The piercing rule first: the person asked about an ear, not about a tattoo.
pub fn _a_teenagers_piercing(memory: &mut Memory, understood: &Understood) -> String {
    let age = _the_age_in_the_line(understood);
    let (he, him, _his, _does) = _the_child_pronouns(memory, understood);
    let lead = match &age {
        Some(age) if crate::py::text::is_digit(age) && age.parse::<i64>().is_ok_and(|n| n >= 16) => format!(
            "At {age} {he} can have lobes and most other placements pierced without a parent's \
             consent, so this one is up to {him}"
        ),
        Some(age) => format!(
            "At {age} a piercing needs you there: most studios want a parent present with ID, \
             and many will only do lobes below 16"
        ),
        None => "Under 16 a piercing usually needs a parent there with ID, and 16 is when lobes \
                 and most other placements stop needing consent"
            .to_string(),
    };
    format!(
        "{lead}. Ring the shop before you go, because the age rules and the placements \
         allowed differ by council, and only a licensed studio is worth using: gloves, \
         sterile sealed needles, a hollow needle rather than a gun for anything but lobes, \
         and written aftercare. A nose ring or a navel heals for months and wants cleaning \
         twice a day, so agree who does that before the appointment. A tattoo is a different \
         rule: 18, no exceptions."
    )
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.family_more:_another_country_is_in_the_line:43", |_: &FnRef, u: &Understood| {
        _another_country_is_in_the_line(u)
    }),
    ("bot.knowledge.family_more:_not_a_nursing_career:547", |_: &FnRef, u: &Understood| _not_a_nursing_career(u)),
    ("bot.knowledge.family_more:_the_child_is_not_grown:197", |_: &FnRef, u: &Understood| _the_child_is_not_grown(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.family_more:_a_teenagers_piercing:1206", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_a_teenagers_piercing(m, u))
    }),
    ("bot.knowledge.family_more:_a_teenagers_tattoo:1178", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_a_teenagers_tattoo(m, u))
    }),
    ("bot.knowledge.family_more:_back_to_school:886", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_back_to_school(m, u))
    }),
    ("bot.knowledge.family_more:_breastfeeding_basics:561", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_breastfeeding_basics(m, u))
    }),
    ("bot.knowledge.family_more:_easter_break:852", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_easter_break(m, u))
    }),
    ("bot.knowledge.family_more:_half_term:861", |_: &FnRef, m: &mut Memory, u: &Understood| Some(_half_term(m, u))),
    ("bot.knowledge.family_more:_nappies_a_day:433", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_nappies_a_day(m, u))
    }),
    ("bot.knowledge.family_more:_police_matter:1029", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_police_matter(m, u))
    }),
    ("bot.knowledge.family_more:_school_holidays_general:902", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_school_holidays_general(m, u))
    }),
    ("bot.knowledge.family_more:_summer_break:814", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_summer_break(m, u))
    }),
    ("bot.knowledge.family_more:_winter_break:834", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_winter_break(m, u))
    }),
];
