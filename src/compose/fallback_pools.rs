//! Port of `bot/compose/fallback_pools.py`: the two pool readers that weigh most, lifted out of
//! `fallback.py` (`_unknown` and `_the_view_asked_for`) with the small readers they use.
//!
//! The names this file imports from `fallback.py` are read from the bundle under this module's
//! own namespace (`bot.compose.fallback_pools.*`); the functions are called in `fallback` and
//! `fallback_chain`, where the Python's `fallback` module lives in Rust.
#![allow(non_snake_case, clippy::all)]

use crate::compose::fallback::{self, _a_plan_doubt_with, _reads_as_a_claim};
use crate::compose::fallback_chain::{self, _choose, _reads, opt_string_of, pool_of, PyRet};
use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.fallback_pools.", stringify!($name))));)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.fallback_pools.", stringify!($name))));)*
    };
}
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.fallback_pools.", stringify!($name))));)*
    };
}

regexes!(
    A_CARE_WORD, A_MONEY_WORD, A_STEP_ASK, BARE_FOLLOW_UP, CRISIS_ASK, HOW_TO_DECIDE, LEAVING_IN_THE_LINE,
    LIFE_DIRECTION, NOT_A_TIPS_ASK, REALISTIC, REALISTIC_ABOUT_ME, REPAIR_BARE_IT, REPAIR_DECISION, REPAIR_PERSON,
    REPAIR_TARIFF, REPAIR_TRADE, REPAIR_WHAT_IT_IS_WORTH, REPAIR_WHETHER, TAG_CAN, TIPS_ASKS, TOO_LATE, VIEW_MIRROR,
    VIEW_NAMES_SOMEBODY, WHO_FOR_IT, MISSED_STEP, MISSED_STEP_PLAN, MISSED_STEP_KEEPS,
);
word_sets!(ANIMALS, BODY_PARTS, CARE_MODULES, EXPERT_WORDS, HEALTH_WORDS, SYMPTOM_WORDS);
pools!(
    MISSED_STEP_LINES, REPAIR_WHETHER_LINES, TIME_RULE, ASKED_IF_I_SAW_LINES, A_PERSON_IN_A_CRISIS,
    BARE_FOLLOW_UP_CARE_REPLIES, BARE_FOLLOW_UP_REPLIES, BARE_PICK_AFTER_NEWS, BODY_IS_NOT_A_MACHINE, CARE_HERE,
    DITHER_OPEN_LINES, HANDED_BACK, HELP_ALREADY, HOW_TO_DECIDE_LINES, NOT_NOTED, POINTING_LINES, REPAIR_RULE,
    SAME_AGAIN, SAME_AGAIN_WITH_THE_ANSWER, TAG_CONFIRM, TAG_UNSURE, TIPS_LINES, UNKNOWN, UNKNOWN_ABOUT_A_PET,
    UNKNOWN_ABOUT_SOMEONE, UNKNOWN_DECIDE, UNKNOWN_HEALTH, UNKNOWN_REALISTIC, UNKNOWN_REALISTIC_CARE,
    UNKNOWN_TOO_LATE, UNKNOWN_WORD, UNKNOWN_WORD_NAMED, UNKNOWN_WORRY, VIEW_AFTER_A_DEATH, VIEW_ASK_LINES,
    VIEW_COMPANY, VIEW_DECIDE, VIEW_HANDBACK, VIEW_LINES, VIEW_MIRROR_LINES, VIEW_NO_SUBJECT, VIEW_ON_GOOD_NEWS,
    WASNT_THERE,
);

/// `waited_for.NOTHING_HELD`, the pool `for_the_ask` gives when nothing was held for the ask.
static NOTHING_HELD: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.waited_for.NOTHING_HELD"));
/// `(NEUTRAL,)`, the pools `_in_a_pool` is asked about.
static NEUTRAL_POOLS: LazyLock<Value> = LazyLock::new(|| {
    Value::Tuple(std::sync::Arc::new(vec![data::get("bot.compose.fallback_pools.NEUTRAL").clone()]))
});

static SENTENCE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s+", 0));
static A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static NOT_A_WORD_CHAR: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9']", 0));

/// `set(re.findall(r"[a-z']+", text))`.
fn az_words(text_: &str) -> Vec<String> {
    A_WORD.findall_str(text_)
}

/// `" " + re.sub(r"[^a-z0-9']", " ", line) + " "`, and its words with the apostrophes taken out.
fn said_and_words(line: &str) -> (String, Vec<String>) {
    let said = format!(" {} ", NOT_A_WORD_CHAR.sub(" ", line));
    let words = text::split(&said.replace('\'', ""));
    (said, words)
}

/// `_choose(...)` where the Python returns the pick straight back (None there is "" here).
fn chosen(options: &[String], memory: &Memory, clean: &str) -> String {
    _choose(options, memory, clean, None).unwrap_or_default()
}

/// `_choose(..., seed_text)`.
fn chosen_seeded(options: &[String], memory: &Memory, clean: &str, seed_text: &str) -> String {
    _choose(options, memory, clean, Some(seed_text)).unwrap_or_default()
}

/// Python: bot/compose/fallback_pools.py:64 `_the_repeat_answer(memory)`
/// The answer the person is asking for again, short enough to say again: the answer above,
/// cut to its first two sentences when the card ran longer.
pub fn _the_repeat_answer(memory: &Memory) -> String {
    let answer = text::squash(&fallback::_the_answer_above(memory));
    if answer.is_empty() {
        return String::new();
    }
    let sentences = SENTENCE_END.split(&answer);
    if sentences.len() > 2 { sentences[..2].join(" ") } else { answer }
}

/// Python: bot/compose/fallback_pools.py:83 `_drafting_pool(clean)`
/// The drafting invite, without asking for what the line already said: when an addressee is
/// already named, the invite asks for the gist and the tone.
pub fn _drafting_pool(clean: &str) -> Vec<String> {
    if crate::compose::no_eyes::names_the_addressee(clean) {
        return UNKNOWN_WORD_NAMED.clone();
    }
    UNKNOWN_WORD.clone()
}

/// Python: bot/compose/fallback_pools.py:96 `_a_care_thread(memory, look_back=3)`
/// True when the last few things the person said were about somebody being looked after.
pub fn _a_care_thread(memory: &Memory, look_back: i64) -> bool {
    if fallback::_thread_is_about_a_pet(memory, 3) {
        return true;
    }
    last_n(&memory.turns, look_back).iter().any(|turn| A_CARE_WORD.is_match(&text::lower(&turn.user)))
}

/// Python: bot/compose/fallback_pools.py:113 `_a_money_thread(memory, look_back=3)`
/// True when the last few things the person said were about money they owe or are owed.
pub fn _a_money_thread(memory: &Memory, look_back: i64) -> bool {
    last_n(&memory.turns, look_back).iter().any(|turn| A_MONEY_WORD.is_match(&text::lower(&turn.user)))
}

/// `seq[-n:]`.
fn last_n<T>(seq: &[T], n: i64) -> &[T] {
    let len = seq.len() as i64;
    let start = if n > 0 { (len - n).max(0) } else { (-n).min(len) };
    &seq[start as usize..]
}

/// Python: bot/compose/fallback_pools.py:122 `_the_rule_for_it_for_the_turn(clean, repaired, memory)`
/// What settles this kind of ask, said before where the rest of it is kept, or None. A thread
/// that is a crisis, a loss or care, a line naming a body or a health word, and a thread about a
/// body stand down.
pub fn _the_rule_for_it_for_the_turn(clean: &str, repaired: &str, memory: &Memory) -> Option<Vec<String>> {
    if fallback::in_a_safety_thread(memory)
        || fallback::_grief_thread(memory)
        || fallback::_in_a_loss_thread(memory)
        || fallback::_in_a_care_thread(memory)
        || fallback::_thread_is_about_the_body(memory, 4)
        || _a_care_thread(memory, 3)
    {
        return None;
    }
    let words: Vec<String> = az_words(&text::lower(clean)).into_iter().chain(az_words(&text::lower(repaired))).collect();
    if words.iter().any(|w| BODY_PARTS.contains(w) || SYMPTOM_WORDS.contains(w) || HEALTH_WORDS.contains(w)) {
        return None;
    }
    let pool = crate::compose::the_rule_for_it::the_rule_for_it(clean, repaired).into_pool_exact();
    if pool.as_ref() == Some(&*TIME_RULE) && _a_money_thread(memory, 3) {
        // A price in a money thread is still a price, but a time in one is a debt being paid
        // down, and that sum is the person's own.
        return None;
    }
    pool
}

/// Python: bot/compose/fallback_pools.py:173 `_a_missed_step(clean, repaired, memory, words)`
/// The pool for a stepped plan with one step missed, or None.
pub fn _a_missed_step(clean: &str, repaired: &str, memory: &Memory, words: &[String]) -> Option<Vec<String>> {
    if !MISSED_STEP.is_match(clean) {
        if repaired == clean || !MISSED_STEP.is_match(repaired) {
            return None;
        }
    }
    if !(MISSED_STEP_PLAN.is_match(clean) || MISSED_STEP_PLAN.is_match(repaired)) {
        return None;
    }
    if MISSED_STEP_KEEPS.is_match(clean) || MISSED_STEP_KEEPS.is_match(repaired) {
        return None;
    }
    if words.iter().any(|w| HEALTH_WORDS.contains(w)) || words.iter().any(|w| BODY_PARTS.contains(w)) {
        return None;
    }
    if fallback::in_a_safety_thread(memory)
        || fallback::_grief_thread(memory)
        || fallback::_in_a_loss_thread(memory)
        || fallback::_thread_is_about_the_body(memory, 4)
        || fallback::_thread_is_about_a_pet(memory, 3)
        || fallback::_thread_is_about_someone_else(memory, &Value::None)
    {
        return None;
    }
    Some(MISSED_STEP_LINES.clone())
}

/// `_reads(PATTERN.search, clean, repaired)`.
fn reads(pattern: &Re, clean: &str, repaired: &str) -> bool {
    _reads(|t: &str| pattern.is_match(t), clean, repaired, false)
}

/// Python: bot/compose/fallback_pools.py:192 `_unknown(understood, memory, clean)`
/// An honest reply to a question nothing can answer: name who can, when it is clear.
///
/// (Python returns whatever `_choose` gives, which is None only for an empty pool; that None is
/// "" here.)
pub fn _unknown(understood: &Understood, memory: &Memory, clean: &str) -> String {
    // The same line with its wrong words put right ("any ideaz" -> "any ideas"), from the
    // reading: typed line first, the repaired one only as a second try.
    let repaired_owned = if understood.plain.is_empty() { clean.to_string() } else { understood.plain.clone() };
    let repaired = repaired_owned.as_str();
    // Their own rota is a place that knows, so the line gets it rather than the hedge.
    if let Some(own_day) = fallback::_their_own_day(clean).filter(|p| !p.is_empty()) {
        return chosen(&own_day, memory, clean);
    }
    let no_eyes = crate::compose::no_eyes::look_at_a_body_lines;
    if let Some(body_lines) = pool_of(no_eyes(clean, Some(repaired), memory)) {
        return chosen(&body_lines, memory, clean);
    }
    if let Some(cannot_see) = pool_of(crate::compose::no_eyes::cannot_see_your_thing(clean)) {
        return chosen(&cannot_see, memory, clean);
    }
    if let Some(described) = pool_of(crate::compose::no_eyes::what_they_described(clean, memory)) {
        return chosen(&described, memory, clean);
    }
    if let Some(in_the_room) = pool_of(crate::compose::no_eyes::a_thing_in_the_room(clean)) {
        return chosen(&in_the_room, memory, clean);
    }
    if crate::compose::no_eyes::writes_the_message(clean) {
        return chosen(&_drafting_pool(clean), memory, clean);
    }
    if fallback::_in_a_crisis_thread(memory)
        && (fallback::_asks_for_a_person_to_talk_to(clean) || CRISIS_ASK.is_match(clean) || WHO_FOR_IT.match_(clean).is_some())
    {
        return chosen(&A_PERSON_IN_A_CRISIS, memory, clean);
    }
    if let Some(lost) = fallback::the_lost_ask_reply(memory, clean, repaired).filter(|s| !s.is_empty()) {
        return lost;
    }
    if fallback::_asks_a_name_back(clean, &understood.words) {
        let given = fallback::_name_given_before(clean, memory);
        if !given.is_empty() {
            let lines: Vec<String> =
                HANDED_BACK.iter().map(|line| fallback_chain::py_format(line, &[("name", &given)])).collect();
            return chosen(&lines, memory, clean);
        }
        return chosen(&NOT_NOTED, memory, clean);
    }
    if let Some(forecast) = fallback::_forecast_pool(clean, repaired, memory).filter(|p| !p.is_empty()) {
        return chosen(&forecast, memory, clean);
    }
    if fallback::reads_as_if_i_saw(clean, memory) {
        return chosen(&ASKED_IF_I_SAW_LINES, memory, clean);
    }
    if fallback::_asked_about_the_day_they_had(clean)
        && !fallback::in_a_safety_thread(memory)
        && !fallback::_grief_thread(memory)
        && !fallback::_in_a_loss_thread(memory)
    {
        return chosen(&WASNT_THERE, memory, clean);
    }
    let mut how_to = HOW_TO_DECIDE.search(clean);
    let mut how_line = clean;
    if how_to.is_none() && repaired != clean {
        how_to = HOW_TO_DECIDE.search(repaired);
        how_line = repaired;
    }
    if let Some(how) = &how_to {
        if !reads(&LIFE_DIRECTION, clean, repaired) && fallback::_nothing_to_choose(&how_line[how.end()..]) {
            let (_, plain) = said_and_words(how_line);
            let named = plain.iter().any(|w| {
                HEALTH_WORDS.contains(w)
                    || EXPERT_WORDS.contains(w)
                    || BODY_PARTS.contains(w)
                    || ANIMALS.contains(w)
                    || ANIMALS.contains(w.trim_end_matches('s'))
            });
            if !named {
                return chosen(&HOW_TO_DECIDE_LINES, memory, clean);
            }
        }
    }
    if fallback::_reads_as_taste(clean, &understood.words) {
        return chosen(&UNKNOWN, memory, clean);
    }
    if reads(&REPAIR_WHETHER, clean, repaired)
        && !reads(&REPAIR_WHAT_IT_IS_WORTH, clean, repaired)
        && !fallback::who_settles(clean, memory).truthy()
        && !fallback::_a_repair_already_spoken_of(memory)
    {
        return chosen(&REPAIR_WHETHER_LINES, memory, clean);
    }
    if reads(&REPAIR_DECISION, clean, repaired)
        && !reads(&REPAIR_TRADE, clean, repaired)
        && !reads(&REPAIR_PERSON, clean, repaired)
        && !(reads(&LEAVING_IN_THE_LINE, clean, repaired) && fallback::_thread_is_about_a_person(memory, &Value::None))
        && !reads(&REPAIR_TARIFF, clean, repaired)
    {
        if fallback::in_a_safety_thread(memory) || fallback::_grief_thread(memory) || fallback::_in_a_loss_thread(memory) {
            return chosen(&CARE_HERE, memory, clean);
        }
        let own_thing = fallback::_names_its_own_thing(clean) || fallback::_names_its_own_thing(repaired);
        let line_words: Vec<String> = az_words(clean).into_iter().chain(az_words(repaired)).collect();
        let no_thing_named = reads(&REPAIR_BARE_IT, clean, repaired) && !fallback::_thread_names_a_machine(memory, &Value::None);
        if !own_thing
            && ((fallback::_thread_is_about_the_body(memory, 4) && !fallback::_thread_names_a_machine(memory, &Value::None))
                || line_words
                    .iter()
                    .any(|w| BODY_PARTS.contains(w) || SYMPTOM_WORDS.contains(w) || HEALTH_WORDS.contains(w)))
        {
            if fallback::_offers_to_fix_themselves(clean) {
                return chosen(&BODY_IS_NOT_A_MACHINE, memory, clean);
            }
            return chosen(&UNKNOWN_HEALTH, memory, clean);
        }
        if !no_thing_named
            && (!fallback::_thread_is_about_a_person(memory, &Value::None)
                || own_thing
                || fallback::_a_thing_was_named(memory, &Value::None))
        {
            return chosen(&REPAIR_RULE, memory, clean);
        }
        return chosen(&UNKNOWN_DECIDE, memory, clean);
    }
    if reads(&TOO_LATE, clean, repaired)
        && fallback::_a_thing_they_started(memory, &Value::None)
        && !fallback::_hedged_recently(memory, 3)
        && !fallback::_thread_is_about_the_body(memory, 4)
    {
        return chosen(&UNKNOWN_TOO_LATE, memory, clean);
    }
    if (_a_plan_doubt_with(|t| REALISTIC.is_match(t), clean, repaired)
        || (_a_plan_doubt_with(|t| REALISTIC_ABOUT_ME.is_match(t), clean, repaired)
            && fallback::_a_plan_in_the_thread(clean, memory)))
        && !fallback::_hedged_recently(memory, 3)
    {
        let pool = if fallback::_the_thread_is_heavy(memory) { &*UNKNOWN_REALISTIC_CARE } else { &*UNKNOWN_REALISTIC };
        return chosen_seeded(pool, memory, clean, &format!("{clean} {}", fallback::_the_line_before(memory)));
    }
    let (mut said, mut words) = said_and_words(clean);
    let waited = crate::compose::waited_for::for_the_ask(clean, repaired, memory);
    let waited_is_nothing = waited.as_ref() == Some(&*NOTHING_HELD);
    if let Some(w) = &waited {
        if !waited_is_nothing && BARE_FOLLOW_UP.match_(clean).is_some() {
            return chosen(w, memory, clean);
        }
    }
    if BARE_FOLLOW_UP.match_(clean).is_some() {
        let pool = if fallback::_the_thread_is_heavy(memory) { &*BARE_FOLLOW_UP_CARE_REPLIES } else { &*BARE_FOLLOW_UP_REPLIES };
        return chosen_seeded(pool, memory, clean, &format!("{clean} {}", fallback::_the_line_before(memory)));
    }
    if let Some(switched) = fallback::_a_switch_off_asked(memory, clean).filter(|p| !p.is_empty()) {
        // Python passes the line and the line before as `clean` here (no seed text).
        return chosen(&switched, memory, &format!("{clean} {}", fallback::_the_line_before(memory)));
    }
    if fallback::_points_at_nothing(clean, memory, repaired) {
        return chosen(&POINTING_LINES, memory, clean);
    }
    let weird = if !fallback::_hedged_recently(memory, 3) { fallback::_the_weird_ask_pool(clean, memory) } else { None };
    if let Some(weird) = weird {
        return chosen(&weird, memory, clean);
    }
    let mut pool = fallback::unknown_pool(clean, &words);
    if pool == *UNKNOWN && repaired != clean {
        (said, words) = said_and_words(repaired);
        pool = fallback::unknown_pool(repaired, &words);
    }
    let is = |pool: &Vec<String>, other: &Vec<String>| pool == other;
    if is(&pool, &UNKNOWN) || is(&pool, &UNKNOWN_DECIDE) {
        if let Some(missed) = _a_missed_step(clean, repaired, memory, &words) {
            return chosen(&missed, memory, clean);
        }
    }
    if is(&pool, &UNKNOWN_WORRY) && fallback::_a_bare_worry(clean) && fallback::_thread_is_about_the_body(memory, 4) {
        // A bare worry in a thread about a body is a health worry.
        pool = UNKNOWN_HEALTH.clone();
    } else if is(&pool, &UNKNOWN_WORRY) && fallback::_a_bare_worry(clean) && fallback::_thread_is_about_a_pet(memory, 3) {
        pool = UNKNOWN_ABOUT_A_PET.clone();
    } else if is(&pool, &UNKNOWN_WORRY)
        && fallback::_a_bare_worry(clean)
        && fallback::_thread_is_about_someone_else(memory, &Value::None)
    {
        pool = UNKNOWN_ABOUT_SOMEONE.clone();
    }
    if is(&pool, &UNKNOWN) || is(&pool, &UNKNOWN_DECIDE) {
        if let Some(w) = waited.as_ref().filter(|w| !w.is_empty()) {
            if is(&pool, &UNKNOWN) || !waited_is_nothing {
                return chosen(w, memory, clean);
            }
        }
    }
    if (is(&pool, &UNKNOWN) || is(&pool, &UNKNOWN_WORRY)) && !fallback::_hedged_recently(memory, 3) {
        if let Some(worry) = fallback::_the_pool_for_a_worry_about_them(clean, memory).filter(|p| !p.is_empty()) {
            return chosen(&worry, memory, clean);
        }
    }
    if is(&pool, &UNKNOWN_WORD) {
        return chosen(&_drafting_pool(clean), memory, clean);
    }
    if fallback::asks_for_more(&said, memory) && !fallback::_hedged_recently(memory, 3) {
        if fallback::_answered_recently(memory) {
            return chosen(&HELP_ALREADY, memory, clean);
        }
    }
    if is(&pool, &UNKNOWN) {
        if let Some(routed) = crate::compose::finding_out::for_the_ask(clean, repaired, understood).filter(|p| !p.is_empty()) {
            if let Some(said) = _choose(&routed, memory, clean, None).filter(|s| !s.is_empty()) {
                return said;
            }
        }
        if let Some(day) = pool_of(crate::compose::for_the_day::for_the_ask(clean, repaired)) {
            if let Some(said) = _choose(&day, memory, clean, None).filter(|s| !s.is_empty()) {
                return said;
            }
        }
    }
    if let Some(named) = fallback::_advisor(memory, clean, &said, &words).filter(|s| !s.is_empty()) {
        return named;
    }
    if let Some(for_the_thread) = fallback::_who_for_the_thread(memory, clean, understood).filter(|p| !p.is_empty()) {
        return chosen(&for_the_thread, memory, clean);
    }
    if _reads(fallback::_is_a_tips_ask, clean, repaired, false) || (understood.help_ask && !NOT_A_TIPS_ASK.is_match(clean)) {
        let tips = fallback::_tips(memory, clean, repaired);
        if !tips.is_empty() {
            return tips;
        }
        if understood.help_ask && !reads(&TIPS_ASKS, clean, repaired) && fallback::_hedged_recently(memory, 3) {
            return chosen(&pool, memory, clean);
        }
        if fallback::_answered_already(memory) {
            return chosen(&HELP_ALREADY, memory, clean);
        }
        return chosen(&TIPS_LINES, memory, clean);
    }
    if understood.tag {
        if fallback::_answered_already(memory) && !TAG_CAN.is_match(clean) {
            return chosen(&TAG_CONFIRM, memory, clean);
        }
        return chosen(&TAG_UNSURE, memory, clean);
    }
    if is(&pool, &UNKNOWN) && fallback::_says_it_already(clean, &understood.words, memory) {
        let answer = _the_repeat_answer(memory);
        if !answer.is_empty() {
            let line = _choose(&SAME_AGAIN_WITH_THE_ANSWER, memory, clean, None)
                .unwrap_or_else(|| panic!("AttributeError: 'NoneType' object has no attribute 'format'"));
            return fallback_chain::py_format(&line, &[("answer", &answer)]);
        }
        return chosen(&SAME_AGAIN, memory, clean);
    }
    let pick = fallback::_bare_pick(clean, memory)
        .filter(|p| !p.is_empty())
        .or_else(|| fallback::_bare_pick(repaired, memory).filter(|p| !p.is_empty()));
    if let Some(pick) = pick {
        return chosen(&pick, memory, clean);
    }
    if fallback::_a_dead_end(clean)
        && !fallback::_in_a_care_thread(memory)
        && !(fallback::_news_above(memory, false) || fallback::_a_subject_above(memory))
    {
        return chosen(&DITHER_OPEN_LINES, memory, clean);
    }
    if let Some(about_a_person) = fallback::_a_question_about_the_person(clean, memory).filter(|s| !s.is_empty()) {
        return about_a_person;
    }
    if is(&pool, &UNKNOWN) && A_STEP_ASK.match_(text::strip(clean)).is_some() && fallback::_news_above(memory, false) {
        return chosen(&BARE_PICK_AFTER_NEWS, memory, clean);
    }
    if is(&pool, &UNKNOWN) {
        if let Some(care) = fallback_chain::_a_care_home_line(Some(understood), memory).filter(|s| !s.is_empty()) {
            return care;
        }
    }
    let price = fallback::_a_price_wants_a_verdict(clean, memory).filter(|p| !p.is_empty());
    if let Some(price) = price {
        if is(&pool, &UNKNOWN) {
            return chosen(&price, memory, clean);
        }
    }
    if is(&pool, &UNKNOWN) {
        let mut mine = crate::compose::fallback_data::the_bots_own_answer(clean).into_pool_exact();
        if mine.is_none() && repaired != clean {
            mine = crate::compose::fallback_data::the_bots_own_answer(repaired).into_pool_exact();
        }
        if let Some(mine) = mine {
            return chosen(&mine, memory, clean);
        }
        if let Some(rule) = _the_rule_for_it_for_the_turn(clean, repaired, memory) {
            return chosen(&rule, memory, clean);
        }
        let crowd = crate::compose::findout::reads_as_crowd(clean);
        let mut place =
            if crowd { None } else { crate::compose::fallback_data::the_place_the_answer_is_kept(clean).into_pool_exact() };
        if place.is_none() && repaired != clean && !crowd {
            place = crate::compose::fallback_data::the_place_the_answer_is_kept(repaired).into_pool_exact();
        }
        if let Some(place) = place {
            return chosen(&place, memory, clean);
        }
    }
    if is(&pool, &UNKNOWN) || is(&pool, &UNKNOWN_WORRY) {
        if let Some(own) = opt_string_of(crate::compose::own_ask::answer(clean, memory)).filter(|s| !s.is_empty()) {
            return chosen(&[own], memory, clean);
        }
    }
    if is(&pool, &UNKNOWN) {
        let mut method = fallback::method_ask_pool(clean, memory);
        if method.is_none() && repaired != clean {
            method = fallback::method_ask_pool(repaired, memory);
        }
        if let Some(method) = method {
            return chosen(&method, memory, clean);
        }
    }
    if let Some(counted) = _a_countdown_in_a_care_thread_(memory, clean, repaired, understood).filter(|s| !s.is_empty()) {
        return counted;
    }
    chosen(&pool, memory, clean)
}

/// Python: bot/compose/fallback_pools.py:686 `_a_countdown_in_a_care_thread_(memory, clean, repaired, understood)`
/// The countdown branch, kept in `fallback_chain`.
pub fn _a_countdown_in_a_care_thread_(memory: &Memory, clean: &str, repaired: &str, understood: &Understood) -> Option<String> {
    fallback_chain::_a_countdown_in_a_care_thread(memory, clean, repaired, understood)
}

/// Python: bot/compose/fallback_pools.py:692 `_the_view_asked_for(clean, memory)`
/// The bot's own view, given back, when that is all the line asks for. Only one turn back, and
/// only when that turn answered something: after a hedge there is no view to give the person.
pub fn _the_view_asked_for(clean: &str, memory: &Memory) -> Option<String> {
    if !fallback::_is_a_view_ask(clean) {
        return None;
    }
    let turns = &memory.turns;
    if turns.is_empty() {
        if VIEW_NAMES_SOMEBODY.is_match(clean) {
            return None;
        }
        return _choose(&VIEW_NO_SUBJECT, memory, clean, None);
    }
    let mut last = turns.len() - 1;
    let mut said = text::strip(&turns[last].bot).to_string();
    if said.is_empty() {
        return None;
    }
    // The turns before `last` that said something, latest last.
    let answered_before = |at: usize| -> Option<usize> {
        (0..at).rev().find(|&i| !text::strip(&turns[i].bot).is_empty())
    };
    if fallback::_only_the_floor(&said) {
        last = answered_before(last)?;
        said = text::strip(&turns[last].bot).to_string();
        if fallback::_nothing_to_stand_behind(&said) {
            return None;
        }
    }
    let receipt_or_neutral =
        |said: &str| crate::memory::ack::reads_as_a_receipt(said) || fallback::_in_a_pool(said, &NEUTRAL_POOLS);
    if !receipt_or_neutral(&said) {
        if let Some(found) = fallback::_the_answer_in_reply(&turns[last].user, &said).as_str().filter(|s| !s.is_empty()) {
            said = found.to_string();
        }
    }
    while receipt_or_neutral(&said) {
        last = answered_before(last)?;
        said = text::strip(&turns[last].bot).to_string();
    }
    let module = turns[last].module.clone().unwrap_or_default();
    let module_value = turns[last].module.as_deref().map(Value::from).unwrap_or(Value::None);
    if fallback::_only_a_reaction(&module_value, &said) {
        return _choose(&VIEW_ON_GOOD_NEWS, memory, clean, None);
    }
    if CARE_MODULES.contains(&module) {
        if module == "loss" || module == "grief" {
            return _choose(&VIEW_AFTER_A_DEATH, memory, clean, None);
        }
        return None;
    }
    let low = text::lower(&said);
    if VIEW_HANDBACK.iter().any(|phrase| low.contains(phrase.as_str())) {
        return _choose(&VIEW_DECIDE, memory, clean, None);
    }
    if fallback::_is_a_hedge(&said) {
        return None;
    }
    let (headline, mirrored) = fallback::the_view_of(&said);
    let has_headline = headline.as_ref().is_some_and(|h| !h.is_empty());
    if VIEW_MIRROR.match_(&low).is_some() && !_reads_as_a_claim(&said) && !has_headline {
        return _choose(&VIEW_COMPANY, memory, clean, None);
    }
    if !_reads_as_a_claim(&said) {
        let first = text::lower(text::strip(&SENTENCE_END.split(text::strip(&said))[0]));
        if !(VIEW_MIRROR.match_(&first).is_some() && has_headline) {
            return None;
        }
    }
    let Some(headline) = headline.filter(|h| !h.is_empty()) else {
        if mirrored {
            return _choose(&VIEW_COMPANY, memory, clean, None);
        }
        return None;
    };
    let lines = if headline.ends_with('?') {
        &*VIEW_ASK_LINES
    } else if mirrored {
        &*VIEW_MIRROR_LINES
    } else {
        &*VIEW_LINES
    };
    let after = fallback::_after_a_colon(&headline);
    let after = after.str();
    let filled: Vec<String> = lines.iter().map(|line| fallback_chain::py_format(line, &[("said", after)])).collect();
    _choose(&filled, memory, clean, None)
}
