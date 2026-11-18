//! Port of `bot/compose/findout.py`: a question whose answer lives somewhere else (opening
//! hours, live travel, what's on, a price) answered with where to look, and a partial answer
//! that names the paper or plate the rest is written on.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets, bundle_strs, bundle_vals, format_named, inline_re};
use crate::compose::pick;
use crate::data::Value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

bundle_strs!("bot.compose.findout"; HOURS, STATUS_PHRASES, TRAVEL_LINES, WHATS_ON_LINES, KICK_OFF_LINES, BUSY_LINES,
    BUSY_NO_PLACE, BUSY_NO_PLACE_AND_CONTEXT, PRICE_LINES, AVAIL_LINES, LOCAL_LINES, TERM_LINES, DELIVER_LINES,
    NEARBY_LINES, OFFICIAL_LINES, RULES_LINES, OPENER, OPENER_TERMS);
bundle_sets!("bot.compose.findout"; A_PLACE, NOT_A_VENUE, THE_PEOPLE, TRAVEL, STATUS, TRAVEL_CLAIM, ON_WORDS, ON_EVENT,
    FREE_COMPOUND, PRICE_THING, NEARBY_SERVICE, OFFICIAL_THING, GAME, ASK_STOP, PET, DIED, THEIR_OWN, THEIR_COPY_ASKS);
bundle_res!("bot.compose.findout"; HOURS_ASK, LATE_ASK, HOW_TO_OPEN, ASKED, OPENS_ASKING, STORY, ROAD, ON_ASK, KICK_OFF,
    WHAT_IS_GOOD, BUSY, CROWD, A_PERSON_SUBJECT, A_WAIT_NOUN, BUSY_ASKS, A_PLACE_IN_A_TURN, A_AND_E, PRICE_ASK, FREE_ASK,
    AVAIL, LOCAL_AREA, LOCAL_ASK, TERM_DATES, NEARBY_AREA, ASKS_LOCAL, NEAREST, DELIVER, HOW_TO, OFFICIAL_ASK, RULES_ASK,
    A_REPORT, A_STATUS_SPENT_ON, THEIR_OWN_ONLINE, NOT_THEIR_STATUS, A_FOLLOW_UP, A_BACK_REFERENCE, A_PLACE_NAMED,
    HOLDS_IT, PAPERWORK, NOT_ABOUT_THE_CONVERSATION, NOT_A_LOOKUP_BUT_A_REQUEST, NOT_ABOUT_THE_VISIT, A_PLAN_ASK,
    A_COOKING_LINE, A_READING_JUST_GIVEN, A_WHY_ASK, A_NOW_OR_NEVER_ASK, A_DECISION_ABOUT_GOING_ON,
    A_METHOD_OR_ADVICE_ASK, A_JUDGEMENT_ASK, NOT_ON_THE_PLATE, NOT_HERE, A_RECEIPT, A_JOB_ON_IT, _DETERMINER);
bundle_vals!("bot.compose.findout"; CARRIERS, KINDS, ASKS, QUESTION, QUESTION_KIND);
inline_re!(A_WORD, r"[a-z0-9'-]+");
inline_re!(A_LEADING_DETERMINER, r"^(?:my|our|your|his|her|their|the|a|an)\s+");

/// `(pattern, name)` rows of KINDS or ASKS.
fn rows(table: &'static Value) -> Vec<(&'static Re, &'static str)> {
    table.items().iter().map(|p| (p.items()[0].as_re().expect("a pattern"), p.items()[1].str())).collect()
}
static KIND_ROWS: LazyLock<Vec<(&'static Re, &'static str)>> = LazyLock::new(|| rows(&KINDS));
static ASK_ROWS: LazyLock<Vec<(&'static Re, &'static str)>> = LazyLock::new(|| rows(&ASKS));

/// `table.get((ask, kind))` for a dict keyed by pairs.
fn by_pair(table: &'static Value, ask: &str, kind: &str) -> Option<Vec<String>> {
    table
        .pairs()
        .iter()
        .find(|(k, _)| k.items().len() == 2 && k.items()[0].str() == ask && k.items()[1].str() == kind)
        .map(|(_, v)| v.strs())
}

fn meets(said: &HashSet<String>, set: &HashSet<String>) -> bool {
    said.iter().any(|w| set.contains(w))
}

/// Python: bot/compose/findout.py:447 `reads_as_a_report(text)`
///
/// True when the line is telling the bot about something rather than asking about it.
pub fn reads_as_a_report(text_: &str) -> bool {
    let said = _said(text_);
    !said.is_empty() && !said.ends_with('?') && A_REPORT.search(&said).is_some()
}

/// Python: bot/compose/findout.py:453 `_words(clean)`
pub fn _words(clean: &str) -> Vec<String> {
    A_WORD.findall_str(clean)
}

/// Python: bot/compose/findout.py:458 `reads_as_crowd(clean)`
///
/// True when the line asks how busy somewhere is - the crowd half of `reads_as_busy`.
pub fn reads_as_crowd(clean: &str) -> bool {
    let t = _said(clean);
    !t.is_empty() && CROWD.search(&t).is_some() && BUSY_ASKS.search(&t).is_some()
}

/// Python: bot/compose/findout.py:470 `reads_as_busy(clean)`
///
/// True when the line asks how busy somewhere is, or how long the wait is.
pub fn reads_as_busy(clean: &str) -> bool {
    let t = _said(clean);
    !t.is_empty() && BUSY.search(&t).is_some() && BUSY_ASKS.search(&t).is_some()
}

/// Python: bot/compose/findout.py:481 `_a_place_a_line_back(memory)`
///
/// The place named in the last thing either side said, if the thread has one in it.
pub fn _a_place_a_line_back(memory: &Memory) -> String {
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
        for side in [&turn.user, &turn.bot] {
            let said = _said(side);
            if !said.is_empty() && A_PLACE_IN_A_TURN.search(&said).is_some() {
                return said;
            }
        }
    }
    String::new()
}

/// Python: bot/compose/findout.py:497 `pool(clean, memory=None)`
///
/// The lines for a question whose answer lives somewhere else, or None.
pub fn pool(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let t = _said(clean);
    if t.is_empty() {
        return None;
    }
    let said: HashSet<String> = _words(&t).into_iter().collect();
    let has = |rx: &Re| rx.search(&t).is_some();
    if reads_as_a_report(&t) {
        return None;
    }
    // How busy it is comes before the opening hours: "is the tip busy today" has "today" in it.
    if has(&BUSY) && has(&BUSY_ASKS) {
        if has(&A_PERSON_SUBJECT) {
            // Somebody's own day. Nothing here queues, so this kind stands down.
            return None;
        }
        if meets(&said, &A_PLACE) || has(&A_PLACE_IN_A_TURN) || has(&A_AND_E) {
            return Some(BUSY_LINES.clone());
        }
        if !_a_place_a_line_back(memory).is_empty() {
            return Some(BUSY_LINES.clone());
        }
        if !has(&A_WAIT_NOUN) {
            // "how busy is it": no place and no word for a wait, so nothing here can be crowded.
            return None;
        }
        if !memory.turns.is_empty() {
            return Some(BUSY_NO_PLACE_AND_CONTEXT.clone());
        }
        return Some(BUSY_NO_PLACE.clone());
    }
    // Hours: a place named in the line, and an ask about when it opens or shuts.
    if (has(&HOURS_ASK) && (meets(&said, &A_PLACE) || said.contains("it"))) || (has(&LATE_ASK) && meets(&said, &A_PLACE)) {
        if !has(&HOW_TO_OPEN) && !meets(&said, &NOT_A_VENUE) && !meets(&said, &THE_PEOPLE) {
            return Some(HOURS.clone());
        }
    }
    // A kick-off, then what's on: both are timetables that change weekly.
    if has(&KICK_OFF) {
        return Some(KICK_OFF_LINES.clone());
    }
    if (meets(&said, &ON_WORDS) || meets(&said, &ON_EVENT)) && has(&ON_ASK) {
        return Some(WHATS_ON_LINES.clone());
    }
    // "what's good at the theatre this month" is the same timetable.
    if meets(&said, &ON_WORDS) && has(&WHAT_IS_GOOD) {
        return Some(WHATS_ON_LINES.clone());
    }
    // Live travel: a way of travelling or a numbered road, and something wrong or due.
    if (meets(&said, &TRAVEL) || has(&ROAD))
        && !has(&HOW_TO)
        && !meets(&said, &TRAVEL_CLAIM)
        && !has(&THEIR_OWN_ONLINE)
        && has(&ASKED)
        && !(has(&STORY) && !has(&OPENS_ASKING))
        && (meets(&said, &STATUS) || STATUS_PHRASES.iter().any(|p| t.contains(p.as_str())))
    {
        return Some(TRAVEL_LINES.clone());
    }
    // What an operator charges for something they run.
    if (has(&PRICE_ASK) && meets(&said, &PRICE_THING)) || (has(&FREE_ASK) && !meets(&said, &FREE_COMPOUND)) {
        return Some(PRICE_LINES.clone());
    }
    if has(&AVAIL) {
        return Some(AVAIL_LINES.clone());
    }
    // A figure about their own town: the council holds it.
    if has(&LOCAL_AREA) && has(&LOCAL_ASK) {
        return Some(LOCAL_LINES.clone());
    }
    if has(&DELIVER) {
        return Some(DELIVER_LINES.clone());
    }
    if has(&TERM_DATES) {
        return Some(TERM_LINES.clone());
    }
    if (has(&NEARBY_AREA) && has(&ASKS_LOCAL)) || (has(&NEAREST) && meets(&said, &NEARBY_SERVICE)) {
        return Some(NEARBY_LINES.clone());
    }
    // The office that issues it: only when a thing to be issued or a rule is named.
    if has(&OFFICIAL_ASK) && !(meets(&said, &PET) && meets(&said, &DIED)) {
        if meets(&said, &OFFICIAL_THING) {
            return Some(OFFICIAL_LINES.clone());
        }
        if has(&RULES_ASK) && !meets(&said, &GAME) && said.iter().any(|w| !ASK_STOP.contains(w)) {
            return Some(RULES_LINES.clone());
        }
    }
    None
}

/// Python: bot/compose/findout.py:604 `pool_over_a_receipt(clean)`
///
/// The lines for a line the composer would otherwise meet with only the flat receipt: a status
/// word spent on a named road or line.
pub fn pool_over_a_receipt(clean: &str) -> Option<Vec<String>> {
    let t = _said(clean);
    if t.is_empty() || A_STATUS_SPENT_ON.search(&t).is_none() {
        return None;
    }
    if reads_as_a_report(&t) {
        return None;
    }
    // The article that sometimes opens it ("the delays on the a38") comes off here.
    let bare = match t.strip_prefix("the ") {
        Some(rest) => text::strip(rest).to_string(),
        None => t.clone(),
    };
    if let Some(found) = pool(&bare, &Memory::new()).filter(|p| !p.is_empty()) {
        return Some(found);
    }
    // A status word spent on a thing is the ask either way - unless the thing is one of theirs.
    if NOT_THEIR_STATUS.search(&bare).is_some() {
        return None;
    }
    Some(TRAVEL_LINES.clone())
}

/// Python: bot/compose/findout.py:642 `more_about_the_journey(clean, memory)`
///
/// The travel answer again, for a follow-up to a turn findout (or another live note) just
/// answered.
pub fn more_about_the_journey(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let t = _said(clean);
    if t.is_empty() || A_FOLLOW_UP.search(&t).is_none() || A_BACK_REFERENCE.search(&t).is_none() {
        return None;
    }
    let last = memory.turns.iter().rev().find(|turn| !turn.bot.is_empty())?.bot.as_str();
    if TRAVEL_LINES.iter().any(|l| l == last) {
        return Some(TRAVEL_LINES.clone());
    }
    // Whatever live note the bot has just given, the follow-up gets that note again.
    for row in crate::data::get("bot.compose.fallback_data.LIVE_KINDS").items() {
        let options = &row.items()[2];
        if !last.is_empty() && options.contains_str(last) {
            return Some(options.strs());
        }
    }
    None
}

/// Python: bot/compose/findout.py:695 `says_where_to_look(reply)`
///
/// True when a reply already names a place that holds what the person asked for.
pub fn says_where_to_look(reply: &str) -> bool {
    let said = _said(reply);
    !said.is_empty() && (A_PLACE_NAMED.search(&said).is_some() || HOLDS_IT.search(&said).is_some())
}

/// Python: bot/compose/findout.py:1095 `_they_have_it(said, ask, determiner)`
///
/// True when the line says the thing is theirs to look at, or the ask is about their own copy.
pub fn _they_have_it(said: &str, ask: &str, determiner: Option<&str>) -> bool {
    if THEIR_COPY_ASKS.contains(ask) {
        return true;
    }
    if determiner.is_some_and(|d| THEIR_OWN.contains(d)) {
        return true;
    }
    A_RECEIPT.search(said).is_some() || A_JOB_ON_IT.search(said).is_some()
}

/// Python: bot/compose/findout.py:1104 `_said(text)`
pub fn _said(text_: &str) -> String {
    text::squash(&text_.to_lowercase())
}

/// What `_the_thing_named` finds: how the person said the thing, its kind, and the determiner
/// in front of it.
pub type Thing = (String, String, Option<String>);

/// Python: bot/compose/findout.py:1112 `_the_thing_named(said)`
///
/// (how the person said the thing, its kind, determiner) or None. The longest match wins, and a
/// paper named in the line beats the subject it is about.
pub fn _the_thing_named(said: &str) -> Option<Thing> {
    let mut best: Option<(String, String, usize)> = None;
    for (pattern, kind) in KIND_ROWS.iter() {
        for m in pattern.finditer(said) {
            let word = m.as_str();
            if best.as_ref().is_none_or(|b| text::len(word) > text::len(&b.0)) {
                best = Some((word.to_string(), kind.to_string(), m.start()));
            }
        }
    }
    if best.is_some() {
        // The first paper named, not the longest: "a letter about my council tax" is about the
        // letter.
        if let Some(paper) = PAPERWORK.finditer(said).into_iter().next() {
            best = Some((paper.as_str().to_string(), "paperwork".to_string(), paper.start()));
        }
    }
    let (word, kind, start) = best?;
    let before = &said[..start];
    let mut said_as = format!("the {word}");
    let found = _DETERMINER.search(before);
    let determiner = found.as_ref().and_then(|f| f.get(1)).map(str::to_string);
    if let Some(f) = &found {
        // "my" reads back as "your": the reply is about the person's own copy of the thing.
        let d = f.str(1);
        let d = match d {
            "my" | "our" => "your",
            other => other,
        };
        said_as = format!("{d} {word}");
    }
    Some((said_as, kind, determiner))
}

/// Python: bot/compose/findout.py:1152 `_the_thing_in_the_thread(memory)`
///
/// The thing the last few lines were about, when this line names none of its own. Only the
/// person's own lines are read, the nearest first, and one more back at most.
pub fn _the_thing_in_the_thread(memory: &Memory) -> Option<Thing> {
    let mut seen = 0;
    for turn in memory.turns.iter().rev() {
        let said = _said(&turn.user);
        if said.is_empty() {
            continue;
        }
        if let Some((word, kind, _)) = _the_thing_named(&said) {
            let word = A_LEADING_DETERMINER.sub("", &word);
            // A thing the thread named is theirs by definition.
            return Some((format!("the {word}"), kind, Some("the".to_string())));
        }
        seen += 1;
        if seen >= 2 {
            return None;
        }
    }
    None
}

/// Python: bot/compose/findout.py:1181 `on_the_thing(clean, memory=None, understood=None)`
///
/// A partial answer that names where the rest lives, or None to keep the hedge.
pub fn on_the_thing(clean: &str, memory: &Memory, understood: &Understood) -> Option<String> {
    let _ = understood;
    let said = _said(clean);
    let has = |rx: &Re| rx.search(&said).is_some();
    if said.is_empty()
        || has(&NOT_HERE)
        || has(&NOT_ABOUT_THE_CONVERSATION)
        || has(&NOT_A_LOOKUP_BUT_A_REQUEST)
        || has(&A_JUDGEMENT_ASK)
        || has(&A_PLAN_ASK)
        || has(&A_COOKING_LINE)
        || has(&A_READING_JUST_GIVEN)
        || has(&A_WHY_ASK)
        || has(&A_METHOD_OR_ADVICE_ASK)
        || has(&A_DECISION_ABOUT_GOING_ON)
        || has(&A_NOW_OR_NEVER_ASK)
        || has(&NOT_ABOUT_THE_VISIT)
    {
        return None;
    }
    let named = _the_thing_named(&said);
    let from_the_thread = named.is_none();
    let (word, kind, determiner) = named.or_else(|| _the_thing_in_the_thread(memory))?;
    let mut ask = ASK_ROWS.iter().find(|(pattern, _)| pattern.search(&said).is_some()).map(|(_, name)| *name);
    if ask.is_none() {
        // A line that names a thing of its own and an ask no carrier settles keeps the hedge.
        if !from_the_thread {
            return None;
        }
        // How a thing is used or kept is not written on it.
        if has(&NOT_ON_THE_PLATE) {
            return None;
        }
        ask = Some("first_step");
    }
    let ask = ask.unwrap();
    // Nobody's copy, no carrier: a general question is the world's, and the hedge says so.
    if !from_the_thread && !_they_have_it(&said, ask, determiner.as_deref()) {
        return None;
    }
    let pool = by_pair(&CARRIERS, ask, &kind).filter(|p| !p.is_empty())?;
    let opener_pool: &Vec<String> = if ask == "get_out" { &OPENER_TERMS } else { &OPENER };
    let opener = format_named(&pick(opener_pool, memory, &said), &[("S", &word)]);
    let carrier = pick(&pool, memory, &said);
    // The question is chosen by the kind as well: a tap has no plate.
    let questions = match by_pair(&QUESTION_KIND, ask, &kind).filter(|q| !q.is_empty()) {
        Some(q) => q,
        None => QUESTION.get(ask).expect("QUESTION has every ask").strs(),
    };
    let question = pick(&questions, memory, &said);
    Some(format!("{opener}. {carrier}. {question}"))
}
