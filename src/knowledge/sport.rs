//! Port of PyBot's `bot/knowledge/sport.py`: the rules people look up, getting better at a
//! game, races and recovery, kit for a wet match, and the guards that keep each card on its sport.
use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.knowledge.sport.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{M}{name}"))
}
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
/// A string global of this module.
fn s(name: &str) -> String {
    g(name).str().to_string()
}

macro_rules! strs {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> = LazyLock::new(|| g(stringify!($name)).strs());)*
    };
}
macro_rules! sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> = LazyLock::new(|| g(stringify!($name)).str_set());)*
    };
}
macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> = LazyLock::new(|| rx(stringify!($name)));)*
    };
}

strs!(
    _CLOCK_HALF,
    _SAID,
    _GOLF_SAID,
    _HURTS,
    _NOT,
    _PROTEIN_TRAINING,
    _CUPS,
    _LEAGUES,
    _BATTING_WORDS,
    _BOWLING_WORDS,
    _BASKETBALL_DRIBBLE_WORDS,
    _COMPARED,
    _A_CARD_WORDS,
    _WET_MATCH_WEATHER,
    _WET_MATCH_WEAR,
    _WET_MATCH_SPORT,
    _WET_MATCH_WATCHING_WORDS,
    _WET_MATCH_I_PLAY,
    _WET_MATCH_PLAYS,
    _WET_MATCH_NOT,
    _GYM_KIT_WORDS,
    _RACE_WORDS,
    _MATCH_WORDS,
    _GIG_WORDS,
    _FIREWORK_WORDS,
);
sets!(_SWIMMING_WORDS, _A_BAT_WORD, _ABOUT_THE_GAME, _BOWL_WORDS, _CRICKET_WORDS, _STUDY_WORDS, _WOUND_WORDS);
regexes!(
    _YEAR,
    _A_SHORTER_RACE,
    _THE_FULL_RACE,
    _A_BLOCK_IN_THE_WATER,
    _A_GAME_WORD,
    _A_GOLF_WORD,
    _SHORT_GAME_JARGON,
    _A_PLAY_ASK,
);

static GROUNDS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    g("GROUNDS").items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect()
});
static _RECOVERY_BY_DISTANCE: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    g("_RECOVERY_BY_DISTANCE").items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect()
});
static _RECOVERY_LOOKBACK: LazyLock<usize> =
    LazyLock::new(|| g("_RECOVERY_LOOKBACK").as_int().unwrap_or(6) as usize);
/// `WIN_YEARS`: each winning country (lower case) and its years, in table order.
static WIN_YEARS: LazyLock<Vec<(String, Vec<i64>)>> = LazyLock::new(|| {
    g("WIN_YEARS")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.items().iter().filter_map(Value::as_int).collect()))
        .collect()
});

/// `" ".join(str(understood.get(key) or "") for key in _SAID).lower()`.
fn said_keys(understood: &Understood) -> String {
    _SAID.iter().map(|k| sh::field(understood, k)).collect::<Vec<_>>().join(" ").to_lowercase()
}

/// The line's words as a set.
fn word_set(understood: &Understood) -> HashSet<&str> {
    understood.words.iter().map(String::as_str).collect()
}

/// False when the line the person typed names a cardboard box, not the sport.
pub fn _not_a_line_about_a_box(understood: &Understood) -> bool {
    static BOX: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bbox(es)?\b|\bcartons?\b|\bcardboard\b", 0));
    static BOXING: LazyLock<Re> = LazyLock::new(|| {
        Re::new(r"\bbox(ing|er|ercise)\b|\bgloves\b|\bspar\b|\bring\b|\bkickbox|\bmuay thai|\bclub\b|\bgym\b|\bclass", 0)
    });
    let text = sh::text_or_clean(understood).to_lowercase();
    if !BOX.is_match(&text) {
        return true;
    }
    BOXING.is_match(&text)
}

/// False when the line is a bus or a train service, not the exercise.
pub fn _not_a_timetable_that_runs(understood: &Understood) -> bool {
    static RUN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\brun(s|ning)?\b|\bran\b", 0));
    if understood.topic.as_deref() != Some("geography-and-travel") {
        return true;
    }
    let text = sh::text_or_clean(understood).to_lowercase();
    !RUN.is_match(&text)
}

/// True when the goggles in the line are the ones in a pool.
pub fn the_goggles_are_for_swimming(understood: &Understood) -> bool {
    if understood.words.iter().any(|w| _SWIMMING_WORDS.contains(w)) {
        return true;
    }
    let said = format!("{} {} {}", understood.clean, understood.plain, understood.text).to_lowercase();
    if _SWIMMING_WORDS.iter().any(|w| said.contains(w.as_str())) {
        return true;
    }
    understood.asks.iter().any(|a| a == "how" || a == "what" || a == "why")
}

/// The goggles fix, with the reason first when the line asks why.
pub fn _goggles_fog(_memory: &Memory, understood: &Understood) -> String {
    let said = understood.text.to_lowercase();
    if said.contains("why") || understood.asks.iter().any(|a| a == "why") {
        return format!("{} {}", s("_GOGGLES_REASON"), s("_GOGGLES_FIX"));
    }
    s("_GOGGLES_FIX")
}

/// False when the line's "put" is the everyday verb and no golf word is beside it.
pub fn _a_put_is_not_a_putt(understood: &Understood) -> bool {
    let said = said_keys(understood);
    let flat = said.replace('-', " ");
    let words = text::split_ref(&flat);
    if words.iter().any(|w| _GOLF_SAID.iter().any(|gw| gw == w)) {
        return true;
    }
    !words.contains(&"put")
}

/// False when "half" in the line is a clock hour and the race is not named at all.
pub fn _a_clock_half_is_not_the_race(understood: &Understood) -> bool {
    let said = text::squash(&said_keys(understood).replace('-', " "));
    if said.contains("half marathon") {
        return true;
    }
    let padded = format!(" {said} ");
    !_CLOCK_HALF.iter().any(|hour| padded.contains(&format!(" {hour} ")))
}

/// "1966; 1958 and 1962; 1958, 1962 and 1970."
pub fn _listed(years: &[i64]) -> String {
    let said: Vec<String> = years.iter().map(i64::to_string).collect();
    if said.len() == 1 {
        return said[0].clone();
    }
    format!("{} and {}", said[..said.len() - 1].join(", "), said[said.len() - 1])
}

/// The winner of the year the line names, the years one country won, or the record.
pub fn _world_cup(_memory: &Memory, understood: &Understood) -> String {
    static YEARS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(19\d\d|20[0-2]\d)\b", 0));
    let clean = understood.clean.as_str();
    let years: Vec<i64> = YEARS.findall_str(clean).iter().filter_map(|y| y.parse().ok()).collect();
    let table = g("WORLD_CUP");
    for &year in &years {
        if let Some((_, row)) = table.pairs().iter().find(|(k, _)| k.as_int() == Some(year)) {
            let r = row.items();
            return format!("In {} it was {}, {}.", year, r[0].str(), r[1].str());
        }
    }
    if let Some(first) = years.first() {
        return format!(
            "The men's World Cup has been held every four years since 1930, apart from 1942 and 1946 for the \
             war, so {first} is not a tournament year."
        );
    }
    let lowered = clean.to_lowercase();
    let mut countries: Vec<&(String, Vec<i64>)> = WIN_YEARS.iter().collect();
    countries.sort_by(|a, b| text::len(&b.0).cmp(&text::len(&a.0)));
    for (country, won) in countries {
        if sh::re_search(&format!(r"\b{}\b", sh::escape(country)), &lowered, 0) {
            let how_many = match won.len() {
                1 => "once".to_string(),
                2 => "twice".to_string(),
                3 => "three times".to_string(),
                4 => "four times".to_string(),
                5 => "five times".to_string(),
                n => format!("{n} times"),
            };
            return format!("{} have won the men's World Cup {}: {}.", text::title(country), how_many, _listed(won));
        }
    }
    "Brazil have won it five times (1958, 1962, 1970, 1994, 2002) and Germany and Italy four each; Argentina \
     won in 2022 and France in 2018, England in 1966."
        .to_string()
}

/// A noise on its own is nothing to fix; a noise that hurts is the pain question.
pub fn _noisy_knee(_memory: &Memory, understood: &Understood) -> String {
    let words = word_set(understood);
    let hurts = _HURTS.iter().any(|w| words.contains(w.as_str())) && !_NOT.iter().any(|w| words.contains(w.as_str()));
    if hurts { s("_KNEE_PAIN") } else { s("_KNEE_NOISY") }
}

/// The everyday protein target, or the one for building muscle, whichever was asked.
pub fn _protein(_memory: &Memory, understood: &Understood) -> String {
    let text = format!("{} {}", understood.clean, understood.norm_text).to_lowercase();
    if sh::any_in(&text, &_PROTEIN_TRAINING) {
        return "To build muscle, aim for about 1.6-2.2g of protein for each kilo you weigh - roughly 110-155g a \
                day at 70kg - spread over three or four meals of 25-40g: chicken, tuna, eggs, Greek yoghurt, \
                lentils or tofu. There is no need for powder if meals get you there, and for healthy kidneys \
                that amount is not a risk. Protein without hard training does little, so the lifting matters \
                as much as the food."
            .to_string();
    }
    "Aim for about 0.75g of protein for each kilo you weigh - roughly 55g a day at 75kg - which is a palm-sized \
     portion at each meal: eggs or yoghurt at breakfast, beans, tuna or chicken at lunch, fish, lentils or mince \
     at dinner. Lifting weights to build muscle takes it higher, to about 1.6g per kilo and over. Most people \
     in the UK already eat enough without powder."
        .to_string()
}

/// The stretch for whichever muscle the line names, asked before or after when it is.
pub fn _stretch(_memory: &Memory, understood: &Understood) -> String {
    let words = word_set(understood);
    let has = |set: &[&str]| set.iter().any(|w| words.contains(w));
    let body = if has(&["hamstring", "hamstrings"]) {
        s("_HAMSTRING_STRETCH")
    } else if has(&["quad", "quads", "thigh", "thighs"]) {
        s("_QUAD_STRETCH")
    } else if has(&["leg", "legs"]) {
        s("_WHOLE_LEG_STRETCH")
    } else {
        s("_CALF_STRETCH")
    };
    if words.contains("before") && words.contains("after") {
        return format!(
            "After, when the muscle is warm, not before: holding a stretch on a cold muscle pulls it more than it \
             loosens it, and five minutes of easy running is a better warm-up than any stretch. {body}"
        );
    }
    body
}

/// The league-versus-cup answer, or nothing when the line compares two competitions.
pub fn _league_and_cup(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = format!("{} {}", understood.clean, understood.words.join(" ")).to_lowercase();
    let cups = _CUPS.iter().filter(|n| said.contains(n.as_str())).count();
    let leagues = _LEAGUES.iter().filter(|n| said.contains(n.as_str())).count();
    if cups + leagues > 1 {
        return None;
    }
    if cups > 0 {
        return None;
    }
    Some(s("_LEAGUE_AND_CUP"))
}

/// The honest line about a score, or nothing when the line asks about another season.
pub fn _cant_see_the_score(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = format!("{} {}", understood.words.join(" "), understood.clean);
    if _YEAR.is_match(&said) {
        return None;
    }
    Some(s("_RESULTS_REPLY"))
}

/// Getting better at basketball, opening with the part the line names.
pub fn _basketball_better(_memory: &Memory, understood: &Understood) -> String {
    let clean = understood.clean.to_lowercase();
    let mut words: HashSet<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    words.extend(text::split(&clean));
    let flat = text::squash(&clean);
    if _BASKETBALL_DRIBBLE_WORDS.iter().any(|w| words.contains(w) || (w.contains(' ') && flat.contains(w.as_str()))) {
        return "Dribble low and hard with your fingertips, head up, and change pace rather than doing tricks: a \
                slow dribble that suddenly speeds up beats a crossover nobody falls for. Pound the ball 50 times \
                with each hand standing still, then walk the length of the court changing hands every three \
                dribbles, then do it at a jog. Keep your body between the ball and the defender, and never pick \
                the ball up until you have decided what you are doing with it."
            .to_string();
    }
    "Shooting is one repeatable motion: elbow under the ball, a small dip, and push up from the legs so the ball \
     leaves your fingertips with backspin. Start a metre from the hoop and make 20 in a row before you step \
     back, then finish every session with 20 free throws while you are tired, because that is when games are \
     decided. Keep the guide hand off the ball and do not let the shooting elbow drift out."
        .to_string()
}

/// False when the "coach" in the line is the person who takes the session.
pub fn _not_the_coach_of_a_team(understood: &Understood) -> bool {
    understood.sense.get("coach").map(String::as_str) != Some("teacher")
}

/// False when the "bat" in the line is not the game the row is about.
pub fn _a_bat_that_is_not_the_game(understood: &Understood) -> bool {
    match understood.sense.get("bat").map(String::as_str) {
        Some("animal") => return false,
        Some("sport") => return true,
        _ => {}
    }
    let mut words: HashSet<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    words.extend(text::split(&understood.clean.to_lowercase()));
    if !words.iter().any(|w| _A_BAT_WORD.contains(w)) {
        return true;
    }
    words.iter().any(|w| _ABOUT_THE_GAME.contains(w))
}

/// The cricket row's one guard: not the other bowl game, and the word is the game or the line is.
pub fn _the_game_and_not_another(understood: &Understood) -> bool {
    _not_another_bowl_game(understood) && _a_bat_that_is_not_the_game(understood)
}

/// False when the line says bowl and nothing else about cricket.
pub fn _not_another_bowl_game(understood: &Understood) -> bool {
    static TOKEN: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9']+", 0));
    let words_line = understood.words.join(" ");
    let text = text::squash(&sh::or(&understood.clean, &words_line).to_lowercase());
    let words: HashSet<String> = TOKEN.findall_str(&text).into_iter().collect();
    if !words.iter().any(|w| _BOWL_WORDS.contains(w)) {
        return true;
    }
    _CRICKET_WORDS.iter().any(|w| words.contains(w) || (w.contains(' ') && text.contains(w.as_str())))
}

/// Getting better at cricket, opening with the half the line actually names.
pub fn _cricket_better(_memory: &Memory, understood: &Understood) -> String {
    let mut words: HashSet<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    words.extend(text::split(&understood.clean.to_lowercase()));
    let batting = _BATTING_WORDS.iter().any(|w| words.contains(w));
    let bowling = _BOWLING_WORDS.iter().any(|w| words.contains(w));
    if bowling && !batting {
        return "Aim at the top of off stump over and over rather than trying a new ball each over: mark a target \
                on the pitch and land 10 out of 12 before you change anything. Run in the same way every time, \
                keep the front arm up and the head still, and use the crease - wide of it for a ball going across, \
                close to it for one coming in. A wicketkeeper or a mate behind the stumps tells you more than a \
                camera does."
            .to_string();
    }
    if batting {
        return "Batting is mostly watching and waiting: watch the ball out of the bowler's hand, keep your head \
                still, play straight and leave anything outside off you do not have to hit. Give every net session \
                a purpose - 20 drives, 20 forward defences, 20 pulls - against a bowler better than the ones you \
                face on a Saturday. Ten minutes of throw-downs a day beats an hour of slogging."
            .to_string();
    }
    "Batting is where most club players lose their place, so start there: watch the ball out of the bowler's \
     hand, keep your head still, play straight and leave anything outside off you do not have to hit. Then the \
     other half - aim at the top of off stump over and over rather than trying a new ball every over - and catch \
     50 high balls a week, because catches win more club games than runs do."
        .to_string()
}

/// False when the line is about revision: a pass on the words, not on their loose forms.
pub fn _not_a_study_line(understood: &Understood) -> bool {
    !understood.words.iter().any(|w| _STUDY_WORDS.contains(&w.to_lowercase()))
}

/// The football boots card, led by when, where, fit or breaking in as the line asks.
pub fn _football_boots(_memory: &Memory, understood: &Understood) -> String {
    let words_line = understood.words.join(" ");
    let text = sh::or(&understood.clean, &words_line).to_lowercase();
    let any = |words: &[&str]| words.iter().any(|w| text.contains(w));
    let surface = "The surface picks the boot: moulded studs (FG) on grass, screw-in studs (SG) when it is wet and \
                   soft, astro or turf trainers (TF) on 3G and 4G, flat indoor shoes for a hall. ";
    let grip = "Astro boots on wet grass give almost no grip, which is how ankles turn.";
    let fit = "Go by fit: try them on in the afternoon with match socks on, leave a thumb's width at the toe, and \
               go half a size up for a child whose feet are still growing. ";
    let new = "Wear a new pair around the house an evening at a time and never in a match first; a leather boot \
               softens within a week. ";
    let where_ = "Buy them anywhere that lets you try them on: a sports shop, a supermarket's own brand, the big \
                  online retailers, or a club's second-hand kit sale, where out-grown pairs go for a few pounds. ";
    let cheap = "Do not spend much on a child's boots - feet grow a size a season - but do buy the right sole for \
                 the pitch, because a cheap boot in the wrong stud is worse than no stud at all. ";
    let when = "Boots are cheapest at the end of a season, in April to June, and again when a new colourway comes \
                out and last year's drop; for a child the club's kit sale just before the season starts is the \
                moment, because that is when the out-grown pairs come in. ";
    if any(&["when", "what month", "which month", "best time", "time to buy", "month"]) {
        return format!("{when}{surface}");
    }
    if any(&[
        "cheap", "cheaper", "cheapest", "afford", "budget", "price", "prices", "cost", "costs", "spend", "sale",
        "second hand", "secondhand", "where", "shop", "shops", "store", "online", "order", "buy from",
    ]) {
        return format!("{where_}{cheap}{surface}{fit}");
    }
    if any(&[
        "tight", "small", "smaller", "size", "sizing", "wide", "narrow", "blister", "blisters", "hurt", "hurts",
        "rubbed", "10 year old", "9 year old", "8 year old", "11 year old", "12 year old", "13 year old",
        "14 year old", "growing",
    ]) {
        return format!("{fit}{surface}{grip}");
    }
    if any(&["break in", "breaking in", "break them in", "new boots", "new pair", "stiff"]) {
        return format!("{new}{surface}{grip}");
    }
    format!("{surface}{grip} Try them on with match socks and leave a thumb's width at the toe.")
}

/// The size of the pitch or court this line names, or None when it names none of them.
pub fn _ground_size(_memory: &Memory, reading: &Understood) -> Option<String> {
    let said = text::squash(&reading.clean);
    if said.is_empty() {
        return Some(GROUNDS[0].1.clone());
    }
    GROUNDS.iter().find(|(words, _)| sh::any_in(&said, words)).map(|(_, t)| t.clone())
}

/// False when the stitch the line is about is one holding a cut together.
pub fn _not_a_stitch_in_a_wound(understood: &Understood) -> bool {
    let words: Vec<String> = if understood.words.is_empty() {
        text::split(&understood.clean)
    } else {
        understood.words.iter().map(|w| w.to_lowercase()).collect()
    };
    !words.iter().any(|w| _WOUND_WORDS.contains(w))
}

/// The line's words joined, with hyphens as spaces.
fn words_said(understood: &Understood) -> String {
    understood.words.join(" ").replace('-', " ")
}

/// True for "is a half marathon longer than a 10k", and only for that shape.
pub fn _a_shorter_race_is_the_question(understood: &Understood) -> bool {
    let said = words_said(understood);
    if !_A_SHORTER_RACE.is_match(&said) {
        return false;
    }
    !_THE_FULL_RACE.is_match(&said)
}

/// False for a line that names the half and is not comparing it with the full race.
pub fn _half_is_not_the_question(understood: &Understood) -> bool {
    let words = &understood.words;
    let said = words_said(understood);
    let has = |w: &str| words.iter().any(|x| x == w);
    let half = has("half") || has("13.1") || has("21.1");
    let compares = has("longer") || sh::any_in(&said, &_COMPARED);
    if half && !_THE_FULL_RACE.is_match(&said) && _A_SHORTER_RACE.is_match(&said) {
        return false;
    }
    compares || !half
}

/// True when the half (not a shorter race) is named: the full-race comparison card's guard.
pub fn _the_half_is_named(understood: &Understood) -> bool {
    let words = &understood.words;
    let said = words_said(understood);
    if _A_SHORTER_RACE.is_match(&said) {
        return false;
    }
    words.iter().any(|w| w == "half" || w == "13.1" || w == "21.1")
}

/// True when the line names a race distance, so its "rest" is a runner's rest.
pub fn _a_distance_is_named(understood: &Understood) -> bool {
    let text = sh::or(&understood.plain, &understood.clean).to_lowercase();
    _RECOVERY_BY_DISTANCE.iter().any(|(phrases, _)| sh::any_in(&text, phrases))
}

/// How long the recovery takes, for the distance named in this line or one just before it.
pub fn _recovery_time(memory: &Memory, understood: &Understood) -> String {
    let text = sh::or(&understood.plain, &understood.clean).to_lowercase();
    for (phrases, line) in _RECOVERY_BY_DISTANCE.iter() {
        if sh::any_in(&text, phrases) {
            return line.clone();
        }
    }
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(*_RECOVERY_LOOKBACK)..].iter().rev() {
        let said = turn.user.to_lowercase();
        for (phrases, line) in _RECOVERY_BY_DISTANCE.iter() {
            if sh::any_in(&said, phrases) {
                return line.clone();
            }
        }
    }
    "Scale it to what you did: a day or two easy after a hard 5k, three or four days after a 10k, a week to ten \
     days after a half marathon, and three to four weeks before anything hard after a marathon. A gym session \
     is about 48 hours for the same muscle group. Soreness shifts faster with a short easy effort than with total \
     rest, and if you are still stiff on the stairs after a week, you went back too soon."
        .to_string()
}

/// The circle throw the line names: hammer, shot or (by default) discus.
pub fn _a_circle_throw(_memory: &Memory, understood: &Understood) -> String {
    let text = understood.clean.to_lowercase();
    if text.contains("hammer") {
        return "Swing it from a low point and keep the arms long: three or four turns that get taller and faster, \
                the body turning first and the arms following, and a high release over the left shoulder. Stay in \
                the circle until the ball lands, and keep the wire clear of the ground on the way round - distance \
                comes from the turns building, not from a big first pull."
            .to_string();
    }
    if text.contains("shot") {
        return "A shot is pushed, not thrown: hold it in the fingers at the base of the hand, tuck it under the \
                chin, and drive off the back leg so the hips lead and the arm follows last. Keep the elbow behind \
                the shot until the push, and stay in the circle - stepping out over the front is a foul."
            .to_string();
    }
    "The throw comes from the legs and the turn, not the arm: hold the discus flat, spin across the circle so the \
     hips lead and the arm follows last, and let it go at about shoulder height with the palm down. Keep the \
     discus close to your body through the turn. Most beginners lose distance by throwing it up and by rushing \
     the spin."
        .to_string()
}

/// True when the line names a card, a match or a sport, so the card row may answer it.
pub fn _a_card_is_named(understood: &Understood) -> bool {
    let line = format!(" {} ", text::squash(&sh::clean_or_text(understood).to_lowercase()));
    _A_CARD_WORDS.iter().any(|word| {
        line.contains(&format!(" {word} "))
            || format!(" {word}") == text::slice(&line, -(text::len(word) as isize) - 1, None)
    })
}

/// True when the line is a fear, a refusal or an ask about going under, not an achievement.
pub fn _a_block_in_the_water(understood: &Understood) -> bool {
    _A_BLOCK_IN_THE_WATER.is_match(sh::text_or_clean(understood))
}

/// False when the line is a game to play rather than golf's short game.
pub fn _a_short_game_to_play(understood: &Understood) -> bool {
    let line = sh::clean_or_text(understood);
    if _A_GOLF_WORD.is_match(line) {
        return true;
    }
    if _SHORT_GAME_JARGON.is_match(line) && !_A_PLAY_ASK.is_match(line) {
        return true;
    }
    !_A_GAME_WORD.is_match(line)
}

/// True when one of these phrases is in the line as a word, not inside a longer word.
pub fn _says<S: AsRef<str>>(clean: &str, phrases: &[S]) -> bool {
    let alts: Vec<String> = phrases.iter().map(|p| sh::escape(p.as_ref())).collect();
    sh::re_search(&format!(r"(?<!\w)(?:{})", alts.join("|")), clean, 0)
}

/// True for a line that asks what to wear, bring or take to a match in bad weather.
pub fn _a_wet_match(reading: &Understood) -> bool {
    let clean = format!(" {} ", reading.clean.to_lowercase());
    if !_says(&clean, &_WET_MATCH_SPORT) {
        return false;
    }
    if !_says(&clean, &_WET_MATCH_WEATHER) {
        return false;
    }
    if !_says(&clean, &_WET_MATCH_WEAR) {
        return false;
    }
    !_says(&clean, &_WET_MATCH_NOT)
}

/// What to wear for a match in the rain, led by the playing or the watching half.
pub fn _kit_for_a_wet_match(_memory: &Memory, understood: &Understood) -> String {
    let clean = format!(" {} ", understood.clean.to_lowercase());
    if _says(&clean, &_WET_MATCH_I_PLAY) {
        return s("_WET_MATCH_PLAYING");
    }
    if _says(&clean, &_WET_MATCH_WATCHING_WORDS) {
        return s("_WET_MATCH_WATCHING");
    }
    if _says(&clean, &_WET_MATCH_PLAYS) {
        return s("_WET_MATCH_PLAYING");
    }
    s("_WET_MATCH_WATCHING")
}

/// What to bring to the gym induction, or what to do with it, whichever the line asked.
pub fn _the_gym_induction(_memory: &Memory, understood: &Understood) -> String {
    let clean = format!(" {} ", understood.clean.to_lowercase());
    if _says(&clean, &_GYM_KIT_WORDS) {
        return s("_GYM_INDUCTION_KIT");
    }
    s("_GYM_INDUCTION")
}

/// The window an event usually starts in, picked by the kind of event the line names.
pub fn _event_start(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = sh::clean_or_text(understood).to_lowercase();
    let kinds: [(&str, &[String]); 4] =
        [("race", &_RACE_WORDS), ("match", &_MATCH_WORDS), ("gig", &_GIG_WORDS), ("fireworks", &_FIREWORK_WORDS)];
    let found: Vec<&str> = kinds.iter().filter(|(_, words)| sh::any_in(&said, words)).map(|(k, _)| *k).collect();
    if found.len() == 1 {
        return g("_EVENT_START").get(found[0]).map(|v| v.str().to_string());
    }
    if !found.is_empty() {
        return Some(s("_EVENT_START_GENERIC"));
    }
    None
}

// --- registry wrappers ------------------------------------------------------------------------

macro_rules! guards {
    ($($w:ident => $f:ident),* $(,)?) => {
        $(fn $w(_f: &FnRef, u: &Understood) -> bool { $f(u) })*
    };
}
guards!(
    g_a_block_in_the_water => _a_block_in_the_water,
    g_a_card_is_named => _a_card_is_named,
    g_a_clock_half_is_not_the_race => _a_clock_half_is_not_the_race,
    g_a_distance_is_named => _a_distance_is_named,
    g_a_put_is_not_a_putt => _a_put_is_not_a_putt,
    g_a_short_game_to_play => _a_short_game_to_play,
    g_a_shorter_race_is_the_question => _a_shorter_race_is_the_question,
    g_a_wet_match => _a_wet_match,
    g_half_is_not_the_question => _half_is_not_the_question,
    g_not_a_line_about_a_box => _not_a_line_about_a_box,
    g_not_a_stitch_in_a_wound => _not_a_stitch_in_a_wound,
    g_not_a_study_line => _not_a_study_line,
    g_not_a_timetable_that_runs => _not_a_timetable_that_runs,
    g_not_the_coach_of_a_team => _not_the_coach_of_a_team,
    g_the_game_and_not_another => _the_game_and_not_another,
    g_the_half_is_named => _the_half_is_named,
    g_the_goggles_are_for_swimming => the_goggles_are_for_swimming,
);

macro_rules! replies {
    ($($w:ident => $f:ident : $kind:tt),* $(,)?) => {
        $(fn $w(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { replies!(@call $kind, $f, m, u) })*
    };
    (@call str, $f:ident, $m:ident, $u:ident) => { Some($f($m, $u)) };
    (@call opt, $f:ident, $m:ident, $u:ident) => { $f($m, $u) };
}
replies!(
    r_a_circle_throw => _a_circle_throw: str,
    r_basketball_better => _basketball_better: str,
    r_cant_see_the_score => _cant_see_the_score: opt,
    r_cricket_better => _cricket_better: str,
    r_event_start => _event_start: opt,
    r_football_boots => _football_boots: str,
    r_goggles_fog => _goggles_fog: str,
    r_ground_size => _ground_size: opt,
    r_kit_for_a_wet_match => _kit_for_a_wet_match: str,
    r_league_and_cup => _league_and_cup: opt,
    r_noisy_knee => _noisy_knee: str,
    r_protein => _protein: str,
    r_recovery_time => _recovery_time: str,
    r_stretch => _stretch: str,
    r_the_gym_induction => _the_gym_induction: str,
    r_world_cup => _world_cup: str,
);

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.sport:_a_block_in_the_water:1200", g_a_block_in_the_water),
    ("bot.knowledge.sport:_a_card_is_named:1166", g_a_card_is_named),
    ("bot.knowledge.sport:_a_clock_half_is_not_the_race:373", g_a_clock_half_is_not_the_race),
    ("bot.knowledge.sport:_a_distance_is_named:1098", g_a_distance_is_named),
    ("bot.knowledge.sport:_a_put_is_not_a_putt:356", g_a_put_is_not_a_putt),
    ("bot.knowledge.sport:_a_short_game_to_play:1232", g_a_short_game_to_play),
    ("bot.knowledge.sport:_a_shorter_race_is_the_question:1005", g_a_shorter_race_is_the_question),
    ("bot.knowledge.sport:_a_wet_match:1307", g_a_wet_match),
    ("bot.knowledge.sport:_half_is_not_the_question:1014", g_half_is_not_the_question),
    ("bot.knowledge.sport:_not_a_line_about_a_box:19", g_not_a_line_about_a_box),
    ("bot.knowledge.sport:_not_a_stitch_in_a_wound:982", g_not_a_stitch_in_a_wound),
    ("bot.knowledge.sport:_not_a_study_line:874", g_not_a_study_line),
    ("bot.knowledge.sport:_not_a_timetable_that_runs:42", g_not_a_timetable_that_runs),
    ("bot.knowledge.sport:_not_the_coach_of_a_team:758", g_not_the_coach_of_a_team),
    ("bot.knowledge.sport:_the_game_and_not_another:793", g_the_game_and_not_another),
    ("bot.knowledge.sport:_the_half_is_named:1043", g_the_half_is_named),
    ("bot.knowledge.sport:the_goggles_are_for_swimming:329", g_the_goggles_are_for_swimming),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.sport:_a_circle_throw:1136", r_a_circle_throw),
    ("bot.knowledge.sport:_basketball_better:721", r_basketball_better),
    ("bot.knowledge.sport:_cant_see_the_score:702", r_cant_see_the_score),
    ("bot.knowledge.sport:_cricket_better:834", r_cricket_better),
    ("bot.knowledge.sport:_event_start:4550", r_event_start),
    ("bot.knowledge.sport:_football_boots:880", r_football_boots),
    ("bot.knowledge.sport:_goggles_fog:348", r_goggles_fog),
    ("bot.knowledge.sport:_ground_size:957", r_ground_size),
    ("bot.knowledge.sport:_kit_for_a_wet_match:1322", r_kit_for_a_wet_match),
    ("bot.knowledge.sport:_league_and_cup:689", r_league_and_cup),
    ("bot.knowledge.sport:_noisy_knee:564", r_noisy_knee),
    ("bot.knowledge.sport:_protein:577", r_protein),
    ("bot.knowledge.sport:_recovery_time:1110", r_recovery_time),
    ("bot.knowledge.sport:_stretch:624", r_stretch),
    ("bot.knowledge.sport:_the_gym_induction:1357", r_the_gym_induction),
    ("bot.knowledge.sport:_world_cup:535", r_world_cup),
];
