//! Port of `bot/modules/greeting.py`: the last resort (PRIORITY -1000). It asks the readers that
//! only answer lines nothing else would, then the composer (`bot/compose/fallback.py`), and then
//! replaces the composer's placeholders - a receipt, a hedge, the floor - with the reading that
//! has something true to say. `respond` is wrapped, as in Python, by the earlier-turn check
//! (`memory.again.replied`) and then the receipt-to-question turn (`a_detail_dropped.turned`).
#![allow(non_snake_case)]

use super::answers::{pools, py_try, PyPool, PyRet};
use crate::compose;
use crate::compose::fallback;
use crate::compose::fallback_chain;
use crate::data::{self, Value};
use crate::memory;
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.modules.greeting.";

fn re(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("{M}{name}"))
}

static A_REFERS_BACK: LazyLock<&'static Re> = LazyLock::new(|| re("A_REFERS_BACK"));
static NOTHING_BEFORE_IT: LazyLock<Vec<String>> = LazyLock::new(|| strs("NOTHING_BEFORE_IT"));
static A_STEP_IN_IT: LazyLock<Vec<String>> = LazyLock::new(|| strs("A_STEP_IN_IT"));
static PUSHING_BACK: LazyLock<&'static Re> = LazyLock::new(|| re("PUSHING_BACK"));
static SIGNING_OFF: LazyLock<Vec<String>> = LazyLock::new(|| strs("SIGNING_OFF"));
static SENTENCE: LazyLock<&'static Re> = LazyLock::new(|| re("SENTENCE"));
static QUOTE_MARK: LazyLock<&'static Re> = LazyLock::new(|| re("QUOTE_MARK"));
static _POINTS_BACK: LazyLock<&'static Re> = LazyLock::new(|| re("_POINTS_BACK"));
static _GO_ON: LazyLock<String> = LazyLock::new(|| data::string(&format!("{M}_GO_ON")));
static THEN_TAIL: LazyLock<&'static Re> = LazyLock::new(|| re("THEN_TAIL"));
static PLAIN: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}PLAIN")));
static RESTART: LazyLock<&'static Re> = LazyLock::new(|| re("RESTART"));
static A_DETAIL_OF_IT: LazyLock<&'static Re> = LazyLock::new(|| re("A_DETAIL_OF_IT"));
static A_FACT_OF_THEIRS: LazyLock<&'static Re> = LazyLock::new(|| re("A_FACT_OF_THEIRS"));
static DONE_A_STEP: LazyLock<&'static Re> = LazyLock::new(|| re("DONE_A_STEP"));
static A_TIME_AHEAD: LazyLock<&'static Re> = LazyLock::new(|| re("A_TIME_AHEAD"));
static A_DOING_SOON: LazyLock<&'static Re> = LazyLock::new(|| re("A_DOING_SOON"));
static PAST_TENSE: LazyLock<&'static Re> = LazyLock::new(|| re("PAST_TENSE"));
static A_NEW_FACT_ABOUT_SOMEONE: LazyLock<&'static Re> = LazyLock::new(|| re("A_NEW_FACT_ABOUT_SOMEONE"));

/// `re.search(r"\b" + step + r"\w*", low)` for each of A_STEP_IN_IT, compiled once.
static A_STEP_RES: LazyLock<Vec<Re>> =
    LazyLock::new(|| A_STEP_IN_IT.iter().map(|step| Re::new(&format!(r"\b{step}\w*"), 0)).collect());
/// `re.findall(r"[a-z']+", ...)`
static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// The composer's pools named by the readers below.
const A_SMALL_DAY: &str = "bot.compose.fallback.A_SMALL_DAY";
const NEUTRAL: &str = "bot.compose.fallback.NEUTRAL";
const ACTS_FOR_ME: &str = "bot.compose.fallback.ACTS_FOR_ME";
static UNKNOWN_HEADS: LazyLock<Vec<String>> =
    LazyLock::new(|| data::strs("bot.compose.fallback.UNKNOWN").iter().map(|l| text::slice(l, 0, Some(40))).collect());
static A_NOD: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.find_a_place.A_NOD"));
static WHO_TO_ASK: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.their_day.WHO_TO_ASK"));

/// `fallback._choose(options, memory, clean)` (seed_text=None).
fn choose(options: &[String], memory: &Memory, clean: &str) -> Option<String> {
    fallback_chain::_choose(options, memory, clean, None)
}

/// `fallback._in_a_pool(text, (POOL, ...))`
fn in_a_pool(reply: &str, names: &[&str]) -> bool {
    fallback::_in_a_pool(reply, &pools(names))
}

/// Python: bot/modules/greeting.py:84 `_a_line_pointing_back_at_nothing(clean, memory)`
/// The pool for an opening line that points at an earlier turn, or None.
pub fn _a_line_pointing_back_at_nothing(clean: &str, memory: &Memory) -> Option<String> {
    if !memory.turns.is_empty() {
        return None;
    }
    if !A_REFERS_BACK.is_match(clean) {
        return None;
    }
    choose(&NOTHING_BEFORE_IT, memory, clean)
}

/// Python: bot/modules/greeting.py:99 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// As bound at the end of greeting.py: `a_detail_dropped.turned(again.replied(respond))`.
pub fn respond(text_: &str, memory: &mut Memory) -> Option<String> {
    // again.replied: the earlier-turn check, applied once at the exit
    let said = {
        let reply = _respond(text_, memory);
        match reply {
            Some(r) if !text::strip(&r).is_empty() => {
                let framed = py_try(|| memory::again::instead(&r, text_, memory).py()).flatten();
                Some(framed.unwrap_or(r))
            }
            other => other,
        }
    };
    // a_detail_dropped.turned: a receipt turned into a question about the thing just said
    let reply_in = said.clone().unwrap_or_default();
    compose::a_detail_dropped::answer_for(text_, memory, &reply_in).py().or(said)
}

/// The body of greeting.py's `respond`, before the two wrappers.
fn _respond(text_: &str, memory: &mut Memory) -> Option<String> {
    let going_on = memory::tried::reply_for(text_, memory).py();
    if going_on.is_some() {
        return going_on;
    }
    let who = compose::who_to_see::reply_for(text_, memory).py();
    if who.is_some() {
        return who;
    }
    let came_back = memory::back_again::reply_for(text_, memory).py();
    if came_back.is_some() {
        return came_back;
    }
    let about_them = memory::their_feeling::reply_for(text_, memory).py();
    if about_them.is_some() {
        return about_them;
    }
    let back_at_nothing = _a_line_pointing_back_at_nothing(&text::squash(&text_.to_lowercase()), memory);
    if back_at_nothing.is_some() {
        return back_at_nothing;
    }
    let against = compose::quantity::the_figure_against_the_answer(text_, memory).py();
    if against.is_some() {
        return against;
    }
    let accepted = fallback::a_doable(text_, memory).py();
    if accepted.is_some() {
        return accepted;
    }
    // `verdict` is read and not used again; the calls stay, as in Python.
    let _verdict = fallback::reads_as_a_verdict(text_, memory) && !fallback::a_verdict_stands_down(memory);
    let a_comment = fallback::reads_as_a_verdict(text_, memory);
    let carried = if a_comment { None } else { memory::detail::answer_for(text_, memory).py() };
    if carried.is_some() {
        return carried;
    }
    let objected = if a_comment { None } else { memory::obstacle::answer_for(text_, memory).py() };
    if objected.is_some() {
        return objected;
    }
    if let Some(described) = compose::no_eyes::what_they_described(text_, memory).pool() {
        return Some(choose(&described, memory, text_).unwrap_or_default());
    }
    let reported = memory::detail::a_report_about_the_thing(text_, memory).py();
    if reported.is_some() {
        return reported;
    }
    let corrected = memory::matters::a_corrected_age(text_, memory).py();
    if corrected.is_some() {
        return corrected;
    }
    let off_to = if a_comment { None } else { compose::moving::reply_for(text_, memory).py() };
    if off_to.is_some() {
        return off_to;
    }
    let tasted = compose::taste::reply_for(text_, memory).py();
    if tasted.is_some() {
        return tasted;
    }
    let retoned = compose::tone_again::retone(memory, text_).py();
    if retoned.is_some() {
        return retoned;
    }
    let serious = compose::safety_doubt::reply_for(text_, memory).py();
    if serious.is_some() {
        return serious;
    }
    let mut reply = fallback_chain::reply_for(text_, memory, true).py().unwrap_or_default();
    let has = |r: &str| !r.is_empty();
    if !a_comment {
        if let Some(in_the_room) = compose::no_eyes::a_thing_in_the_room_over_a_placeholder(text_, &reply).pool() {
            return Some(choose(&in_the_room, memory, text_).unwrap_or_default());
        }
        if compose::no_eyes::_is_a_placeholder(&reply) || fallback::is_a_hedge(&reply) {
            // both pools are read before either is tried, as Python builds the tuple first
            let handed = compose::no_eyes::a_contact_they_handed_over(text_).pool();
            let offered = compose::no_eyes::a_recording_they_offer(text_).pool();
            if let Some(pool) = handed.or(offered) {
                return Some(choose(&pool, memory, text_).unwrap_or_default());
            }
        }
    }
    if has(&reply) && (in_a_pool(&reply, &[A_SMALL_DAY]) || memory::ack::reads_as_a_receipt(&reply)) {
        let of_mine = compose::remark::a_thing_of_theirs_reply_for(text_, &text::split(text_), memory).py();
        if of_mine.is_some() {
            return of_mine;
        }
    }
    let judged_line = compose::said_about_me::reads_as_reported_judgement(text_);
    let nothing_yet = !has(&reply)
        || fallback::_nothing_to_stand_behind(&reply)
        || (judged_line && compose::said_about_me::speaks_to_a_self_judgement(&reply));
    if !a_comment && nothing_yet {
        let judged = compose::said_about_me::answer_for(text_, memory)
            .py()
            .or_else(|| compose::said_about_me::answer_followup(text_, memory).py());
        if judged.is_some() {
            return judged;
        }
    }
    let not_a_body = compose::breakup::answer_for(text_, &reply, memory).py();
    if not_a_body.is_some() {
        return not_a_body;
    }
    let keeping_going = compose::griefplea::reply_for(text_, &reply, memory).py();
    if keeping_going.is_some() {
        return keeping_going;
    }
    reply = first_line_points_back_at_nothing(&reply, memory);
    let years_back = compose::goodnews::reply_for_a_year_ago(text_, &reply, memory).py();
    if years_back.is_some() {
        return years_back;
    }
    let stretch = compose::stretches::the_stretch_said_back(text_, memory, &reply).py();
    if stretch.is_some() {
        return stretch;
    }
    if !a_comment && has(&reply) && fallback::_nothing_to_stand_behind(&reply) {
        let refused = memory::obstacle::a_move_refused(text_, memory).py();
        if refused.is_some() {
            return refused;
        }
    }
    if has(&reply) && (fallback::_nothing_to_stand_behind(&reply) || fallback::_is_a_hedge(&reply)) {
        let cannot_use = memory::obstacle::answer_for_a_thing_they_cannot_use(text_, memory).py();
        if cannot_use.is_some() {
            return cannot_use;
        }
    }
    let rest = the_step_they_have_not_had(text_, memory, &reply);
    if !rest.is_empty() {
        return Some(rest);
    }
    if has(&reply)
        && (fallback::_nothing_to_stand_behind(&reply) || fallback::_is_a_hedge(&reply) || in_a_pool(&reply, &[A_SMALL_DAY]))
    {
        let verdict_answer = memory::the_verdict::answer_for(text_, memory).py();
        if verdict_answer.is_some() {
            return verdict_answer;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let again_asked = memory::repeat::the_same_question_again(text_, memory).py();
        if again_asked.is_some() {
            return again_asked;
        }
        let outcome = fallback::an_outcome_asked_about_someone(text_, memory).py();
        if outcome.is_some() {
            return outcome;
        }
        let figure = memory::repeat::the_figure_asked(text_, memory).py();
        if figure.is_some() {
            return figure;
        }
        let figure = memory::repeat::the_figure_they_gave(text_, memory).py();
        if figure.is_some() {
            return figure;
        }
        let worth = compose::is_it_worth_it::is_it_worth_it(text_, memory, &reply).py();
        if worth.is_some() {
            return worth;
        }
    }
    if let Some(back) = memory::repeat::asked_back(text_, memory).py() {
        if !memory::repeat::carries_the_figure(&Value::from(back.as_str()), &reply) {
            return Some(back);
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let late = memory::too_late::answer_for(text_, memory).py();
        if late.is_some() {
            return late;
        }
    }
    if has(&reply) && (fallback::_is_a_hedge(&reply) || in_a_pool(&reply, &[NEUTRAL])) {
        let checks = compose::checking::the_checks_already_named(text_, memory).py();
        if checks.is_some() {
            return checks;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let theirs = compose::how_it_went::answer_for(text_, memory).py();
        if theirs.is_some() {
            return theirs;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let own = compose::own_facts::answer_for(text_, memory).py();
        if own.is_some() {
            return own;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let past = compose::own_facts::about_earlier_chats(text_, memory).py();
        if past.is_some() {
            return past;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let taught = compose::own_facts::learns_from_the_chat(text_, memory).py();
        if taught.is_some() {
            return taught;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let plan = compose::plans::answer_for(text_, memory).py();
        if plan.is_some() {
            return plan;
        }
    }
    if has(&reply)
        && (fallback::_is_a_hedge(&reply)
            || fallback::_nothing_to_stand_behind(&reply)
            || compose::setting_off::is_a_pointer(&reply))
    {
        let off = compose::setting_off::answer_for(text_, memory).py();
        if off.is_some() {
            return off;
        }
    }
    if has(&reply) && (fallback::_is_a_hedge(&reply) || fallback::_nothing_to_stand_behind(&reply)) {
        let gave = if fallback::_thread_is_about_the_body(memory, 4) || fallback::_the_thread_is_heavy(memory) {
            None
        } else {
            compose::how_it_went::the_answer_they_gave(text_, memory).py()
        };
        if gave.is_some() {
            return gave;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let owned = compose::rule::answer_for(text_, memory).py();
        if owned.is_some() {
            return owned;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let named = compose::checking::answer_for(text_, memory).py();
        if named.is_some() {
            return named;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let glad = compose::pleased::answer_for(text_, memory).py();
        if glad.is_some() {
            return glad;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let thread = fallback::the_threads_answer(text_, memory).py();
        if thread.is_some() {
            return thread;
        }
    }
    if has(&reply) && (fallback::_is_a_hedge(&reply) || text::starts_with_any(&reply, &WHO_TO_ASK)) {
        let care = fallback::_the_thread_is_heavy(memory);
        if let Some(asked_back) = compose::their_day::pool_for(text_, care).pool() {
            return choose(&asked_back, memory, text_);
        }
    }
    if has(&reply) && (fallback::_is_a_hedge(&reply) || fallback::_nothing_to_stand_behind(&reply)) {
        let after = compose::the_turn_after::answer_for(text_, memory).py();
        if after.is_some() {
            return after;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let own = compose::their_day::own_fact_for(text_, memory).py();
        if own.is_some() {
            return own;
        }
        // (inside the same block in Python: the comment above it does not end the `if`)
        let parts = compose::no_page::reply_for(text_, memory).py();
        if parts.is_some() {
            return parts;
        }
    }
    if has(&reply) && (fallback::_is_a_hedge(&reply) || fallback::_is_an_ask_which_bit(&reply)) {
        let course = compose::the_course_of_it::reply_for(text_, memory).py();
        if course.is_some() {
            return course;
        }
    }
    let deciding = compose::liveask::answer_for(text_, memory, &reply).py();
    if deciding.is_some() {
        return deciding;
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let one_half = compose::missing_half::the_missing_half(text_, memory).py();
        if one_half.is_some() {
            return one_half;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        if !a_comment {
            let plain = memory::detail::the_plain_ask(text_, memory).py();
            if plain.is_some() {
                return plain;
            }
        }
        let on_screen = compose::onscreen::the_answer_on_screen(text_, memory).py();
        if on_screen.is_some() {
            return on_screen;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let asked = memory::asked_about::the_answer_this_chat_gave(text_, memory).py();
        if asked.is_some() {
            return asked;
        }
    }
    if has(&reply) {
        let not_good = compose::rotten_luck::answer_for(text_, memory, &reply).py();
        if not_good.is_some() {
            return not_good;
        }
    }
    let low_day = compose::nottheirbest::answer_for(text_, &reply, memory).py();
    if low_day.is_some() {
        return low_day;
    }
    if has(&reply) && UNKNOWN_HEADS.contains(&text::slice(&reply, 0, Some(40))) {
        let only = compose::only_they_know::answer_for(text_, memory).py();
        if only.is_some() {
            return only;
        }
    }
    if has(&reply)
        && (fallback::_is_a_hedge(&reply)
            || compose::a_document_lives_with_an_office::a_map_app_line(&reply)
            || compose::a_document_lives_with_an_office::a_lost_record(text_, &reply))
    {
        let held = compose::a_document_lives_with_an_office::answer_for(text_, memory, &reply).py();
        if held.is_some() {
            return held;
        }
    }
    if has(&reply) && (fallback::_is_a_hedge(&reply) || A_NOD.contains(&reply)) {
        let here = if compose::find_a_place::already_a_route(&reply) {
            None
        } else {
            compose::find_a_place::answer_for(text_, memory).py()
        };
        if here.is_some() {
            return here;
        }
    }
    if has(&reply) {
        let handed = compose::onmybehalf::answer_for(text_, memory, &reply).py();
        if handed.is_some() {
            return handed;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let judged = compose::a_place_nobody_can_see::answer_for(text_, memory, &reply).py();
        if judged.is_some() {
            return judged;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let known = compose::what_i_do_know::reply_for(text_, memory, &reply).py();
        if known.is_some() {
            return known;
        }
    }
    if memory::repeat::answers_nothing(&reply) {
        let part = memory::repeat::the_part_asked(text_, memory).py();
        if part.is_some() {
            return part;
        }
        if !memory::repeat::the_advice_was_to_get_seen(memory) {
            let knows = memory::repeat::what_it_knows(text_, memory).py();
            if knows.is_some() {
                return knows;
            }
        }
    }
    let clean = match memory.understood.as_ref().map(|u| u.clean.as_str()) {
        Some(c) if !c.is_empty() => c.to_string(),
        _ => text::squash(&text_.to_lowercase()),
    };
    let thinner = has(&reply)
        && (in_a_pool(&reply, &[NEUTRAL])
            || (fallback::_is_a_hedge(&reply)
                && (compose::findout::reads_as_busy(&clean) || !compose::findout::says_where_to_look(&reply))));
    if thinner {
        let mut elsewhere = compose::findout::pool(&clean, memory)
            .pool()
            .or_else(|| compose::findout::more_about_the_journey(&clean, memory).pool());
        if elsewhere.is_none() && in_a_pool(&reply, &[NEUTRAL]) {
            elsewhere = compose::findout::pool_over_a_receipt(&clean).pool();
        }
        if let Some(elsewhere) = elsewhere {
            return choose(&elsewhere, memory, &clean);
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let understood = memory.understood.clone().unwrap_or_default();
        let looked = compose::findout::on_the_thing(text_, memory, &understood).py();
        if looked.is_some() {
            return looked;
        }
    }
    if has(&reply)
        && (fallback::_is_a_hedge(&reply) || compose::gap::names_only_a_page(&reply))
        && !compose::gap::names_the_paper(&reply)
    {
        if let Some(gap_lines) = compose::gap::pool(&clean, &Memory::new()).pool() {
            return choose(&gap_lines, memory, &clean);
        }
    }
    let mut a_place = fallback::_is_a_hedge(&reply)
        || in_a_pool(&reply, &[NEUTRAL, A_SMALL_DAY])
        || memory::ack::reads_as_a_receipt(&reply);
    if !a_place && compose::hands::writes_the_words(text_) {
        a_place = in_a_pool(&reply, &[ACTS_FOR_ME]);
    }
    if a_place {
        let hands_reply = compose::hands::reply_for(text_, memory).py();
        if hands_reply.is_some() {
            return hands_reply;
        }
    }
    if has(&reply) && fallback::_is_a_hedge(&reply) {
        let own_life = compose::their_life::reply_for(text_, memory).py();
        if own_life.is_some() {
            return own_life;
        }
    }
    // the composer's reply is a str; None (no line at all) comes back as ""
    if reply.is_empty() { None } else { Some(reply) }
}

/// Python: bot/modules/greeting.py:911 `first_line_points_back_at_nothing(reply, memory)`
/// The reply, with any "where were we" taken out when nothing has been said before it.
pub fn first_line_points_back_at_nothing(reply: &str, memory: &Memory) -> String {
    if text::strip(reply).is_empty() {
        return reply.to_string();
    }
    if memory.turns.iter().any(|one| !text::strip(&one.bot).is_empty()) {
        return reply.to_string();
    }
    let sentences: Vec<String> =
        SENTENCE.split(text::strip(reply)).into_iter().filter(|one| !text::strip(one).is_empty()).collect();
    match sentences.last() {
        Some(last) if _POINTS_BACK.match_(text::strip(last)).is_some() => {}
        _ => return reply.to_string(),
    }
    let kept = sentences[..sentences.len() - 1]
        .iter()
        .filter(|one| !text::strip(one).is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    if kept.is_empty() {
        return _GO_ON.clone();
    }
    if text::split_ref(&kept).len() > 6 { kept } else { format!("{kept} {}", *_GO_ON) }
}

/// Python: bot/modules/greeting.py:937 `sentences_of(answer)`
/// The answer's sentences, with a boundary inside an open quotation left alone.
pub fn sentences_of(answer: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0;
    for m in SENTENCE.finditer(answer) {
        let piece = &answer[start..m.start()];
        if QUOTE_MARK.findall(piece).len() % 2 == 1 {
            continue; // a full stop inside a quotation the answer has not closed yet
        }
        out.push(text::strip(piece).to_string());
        start = m.end();
    }
    out.push(text::strip(&answer[start..]).to_string());
    out.into_iter().filter(|one| !one.is_empty()).collect()
}

/// Python: bot/modules/greeting.py:978 `_a_step_of_this_answer(text, answer)`
/// True when the step they report is one of the answer's own, so its next step is theirs.
pub fn _a_step_of_this_answer(text_: &str, answer: &str) -> bool {
    let low_text = text_.to_lowercase();
    let low_answer = answer.to_lowercase();
    let words: Vec<String> = WORDS
        .findall_str(&low_text)
        .into_iter()
        .filter(|w| text::len(w) > 2 && !PLAIN.contains(w))
        .collect();
    let theirs: HashSet<String> = WORDS.findall_str(&low_answer).iter().map(|w| text::slice(w, 0, Some(5))).collect();
    let restarting = RESTART.is_match(&low_text) && RESTART.is_match(&low_answer);
    for word in &words {
        if theirs.contains(&text::slice(word, 0, Some(5))) || (restarting && RESTART.is_match(word)) {
            continue;
        }
        return false;
    }
    !words.is_empty()
}

/// Python: bot/modules/greeting.py:1048 `_a_plan_for_later(text)`
/// True when the line says what they will do and when, rather than how things are.
pub fn _a_plan_for_later(text_: &str) -> bool {
    if DONE_A_STEP.is_match(text_) {
        return false;
    }
    if !A_TIME_AHEAD.is_match(text_) {
        return false;
    }
    for doing in A_DOING_SOON.finditer(text_) {
        if !PAST_TENSE.is_match(doing.as_str()) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/greeting.py:1066 `_answers_the_line(text, answer, turns=())`
/// True when the line is a detail of that answer rather than news of its own.
pub fn _answers_the_line(text_: &str, answer: &str, turns: &[Turn]) -> bool {
    if A_DETAIL_OF_IT.is_match(text_) || DONE_A_STEP.is_match(text_) {
        return true;
    }
    if !A_FACT_OF_THEIRS.is_match(text_) {
        return false;
    }
    let named: HashSet<String> = fallback::content_words(text_).into_iter().collect();
    let answer_words: HashSet<String> = fallback::content_words(answer).into_iter().collect();
    if named.intersection(&answer_words).next().is_some() {
        return true;
    }
    let asked_before = turns.last().map(|one| one.user.clone()).unwrap_or_default();
    let before_words: HashSet<String> = fallback::content_words(&asked_before).into_iter().collect();
    named.intersection(&before_words).next().is_none()
}

/// Python: bot/modules/greeting.py:1098 `_said_back(text, said)`
/// True when the reply is the person's own words back, with no claim added.
pub fn _said_back(text_: &str, said: &str) -> bool {
    let mine: HashSet<String> = fallback::_the_words(text_).into_iter().collect();
    let theirs: HashSet<String> = fallback::_the_words(said).into_iter().collect();
    if mine.len() < 2 || theirs.is_empty() {
        return false;
    }
    if fallback::_is_a_hedge(said) {
        return false;
    }
    let shared = mine.intersection(&theirs).count();
    shared >= 2 && shared as f64 / mine.len() as f64 >= 0.6
}

/// Python: bot/modules/greeting.py:1121 `the_step_they_have_not_had(text, memory, said)`
/// The part of the last answer the person has not had yet, or "" when there is none.
pub fn the_step_they_have_not_had(text_: &str, memory: &Memory, said: &str) -> String {
    let said = text::strip(said);
    let an_echo = _said_back(text_, said);
    let a_step_done = DONE_A_STEP.is_match(text_) && fallback_chain::_a_small_day(text_, memory);
    if said.is_empty() || !(an_echo || a_step_done) {
        return String::new();
    }
    if fallback::is_a_forecast(said) {
        return String::new();
    }
    if fallback::is_a_forecast(said) {
        return String::new();
    }
    if said.contains('?') && !(a_step_done && !an_echo) {
        return String::new();
    }
    let words = WORDS.findall_str(&text_.to_lowercase());
    if words.is_empty() || words.len() > 12 || text_.contains('?') {
        return String::new();
    }
    let turns = &memory.turns;
    let answer = turns.last().map(|one| text::strip(&one.bot).to_string()).unwrap_or_default();
    if answer.is_empty() || !_answers_the_line(text_, &answer, turns) {
        return String::new();
    }
    if _the_answer_again(&answer, turns) {
        return String::new();
    }
    if PUSHING_BACK.is_match(text_) {
        return String::new();
    }
    if A_NEW_FACT_ABOUT_SOMEONE.is_match(text_) {
        return String::new();
    }
    if _a_plan_for_later(text_) {
        return String::new();
    }
    if fallback::reads_as_a_verdict(text_, memory) || fallback::a_verdict_stands_down(memory) {
        return String::new();
    }
    if a_step_done && !an_echo && !_a_step_of_this_answer(text_, &answer) {
        return String::new();
    }
    let sentences = sentences_of(&answer);
    let mut candidates: Vec<String> = Vec::new();
    if sentences.len() > 1 && !(a_step_done && !an_echo) {
        candidates.push(sentences[1..sentences.len().min(3)].join(" "));
    }
    let did_a_step = DONE_A_STEP.is_match(text_);
    if let Some(m) = THEN_TAIL.finditer(&answer).into_iter().find(|m| did_a_step || _said_back(text_, &answer[..m.start()])) {
        let after = text::strip(&answer[m.end()..]);
        if !after.is_empty() {
            candidates.push(text::upper_first(after));
        }
    }
    for leftover in candidates {
        let low = leftover.to_lowercase();
        if text::split_ref(&leftover).len() < 6 || text::starts_with_any(&low, &SIGNING_OFF) {
            continue;
        }
        if !memory::detail::stands_alone(&leftover) {
            continue;
        }
        if text::rstrip(&leftover).ends_with('?') {
            continue;
        }
        if !A_STEP_RES.iter().any(|step| step.is_match(&low)) {
            continue;
        }
        return leftover;
    }
    String::new()
}

/// Python: bot/modules/greeting.py:1229 `_the_answer_again(answer, turns)`
/// True when the last reply is an earlier answer of the bot's, said again.
pub fn _the_answer_again(answer: &str, turns: &[Turn]) -> bool {
    let low = text::squash(&answer.to_lowercase());
    let earlier_turns = if turns.is_empty() { &turns[..0] } else { &turns[..turns.len() - 1] };
    for one in earlier_turns {
        let earlier = text::squash(&one.bot.to_lowercase());
        if text::split_ref(&earlier).len() >= 6 && low.contains(&earlier) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/greeting.py:1246 `takes_as_it_comes(text, memory)` (also bound as
/// `TAKES_AS_IT_COMES`)
/// True when the composer's own line is the answer, whatever else proposed.
pub fn takes_as_it_comes(text_: &str, memory: &Memory) -> bool {
    fallback::reads_as_about_itself(text_, memory)
        || fallback::reads_as_a_tight_week(text_, memory)
        || fallback::a_continuation_of_the_tight_week(memory, text_)
        || fallback::reads_as_small_talk(text_, memory)
        || compose::quantity::the_figure_against_the_answer(text_, memory).py().is_some()
        || compose::moving::reads_as_a_move_cheered(text_)
        || compose::who_to_see::replies_to_a_who_to_see(text_, memory)
        // `fallback.TAKES_AS_IT_COMES` takes the memory mutably in its contract; this one is read
        // with the arbiter's memory, which it is handed as it is.
        || fallback_chain::takes_as_it_comes(text_, &mut memory.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values from `harness/pycall.py bot.modules.greeting ...`.
    #[test]
    fn sentences_and_plans() {
        assert_eq!(
            sentences_of("Say it: 'I love that. Could we?' Then go. Fine."),
            vec!["Say it: 'I love that. Could we?' Then go.".to_string(), "Fine.".to_string()]
        );
        assert!(_a_plan_for_later("im taking it back tomorrow"));
        assert!(!_a_plan_for_later("i called them on monday"));
        assert!(_a_step_of_this_answer("ive turned it off and on", "Switch the printer off and on at the wall, then check the queue."));
        assert!(!_a_step_of_this_answer("i checked the football scores", "Check the boiler pressure, then bleed the radiators."));
    }
}
