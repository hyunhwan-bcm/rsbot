//! Port of `bot/compose/badnews.py`.
//!
//! GENERATED SKELETON: every function is a stub until its workstream ports it. The
//! signatures are the contract other files call against; keep them.
#![allow(unused_variables, unused_imports, non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::knowledge::entry::Entry;
use crate::py::re::Re;
use crate::runtime::Candidate;
use crate::state::{Memory, Turn};
use crate::understood::Understood;
use std::sync::LazyLock;

/// Python: bot/compose/badnews.py:289 `_without_the_noun_leave(said)`
pub fn _without_the_noun_leave(said: &str) -> String {
    crate::unported!("bot.compose.badnews:_without_the_noun_leave")
}

/// Python: bot/compose/badnews.py:294 `_a_leaving_at_all(said)`
pub fn _a_leaving_at_all(said: &str) -> bool {
    crate::unported!("bot.compose.badnews:_a_leaving_at_all")
}

/// Python: bot/compose/badnews.py:305 `reads_as_a_leaving_already_gone(clean)`
pub fn reads_as_a_leaving_already_gone(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_a_leaving_already_gone")
}

/// Python: bot/compose/badnews.py:315 `reads_as_a_reported_leaving(clean)` (never called on the trace corpus)
pub fn reads_as_a_reported_leaving(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_a_reported_leaving")
}

/// Python: bot/compose/badnews.py:333 `reads_as_the_leaving(clean)`
pub fn reads_as_the_leaving(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_the_leaving")
}

/// Python: bot/compose/badnews.py:659 `reads_as_an_award_lost(clean)` (never called on the trace corpus)
pub fn reads_as_an_award_lost(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_an_award_lost")
}

/// Python: bot/compose/badnews.py:668 `reads_as_a_sacking(clean)`
pub fn reads_as_a_sacking(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_a_sacking")
}

/// Python: bot/compose/badnews.py:681 `reads_as_news_with_a_name(clean)` (never called on the trace corpus)
pub fn reads_as_news_with_a_name(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_news_with_a_name")
}

/// Python: bot/compose/badnews.py:692 `reads_as_heavy_news(clean)`
pub fn reads_as_heavy_news(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_heavy_news")
}

/// Python: bot/compose/badnews.py:756 `reads_as_a_closed_door(clean)`
pub fn reads_as_a_closed_door(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_a_closed_door")
}

/// Python: bot/compose/badnews.py:785 `_nearly(word)`
pub fn _nearly(word: &str) -> bool {
    crate::unported!("bot.compose.badnews:_nearly")
}

/// Python: bot/compose/badnews.py:924 `reads_as_a_bare_ailment(clean)`
pub fn reads_as_a_bare_ailment(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_a_bare_ailment")
}

/// Python: bot/compose/badnews.py:1018 `_someone_elses_symptoms(who, sick=CARED_SICK, owner='my|our', named=None, generic=True, body_only=False, part=None)` (never called on the trace corpus)
pub fn _someone_elses_symptoms(who: &str, sick: &Value, owner: &Value, named: &Value, generic: &Value, body_only: &Value, part: &str) -> Value {
    crate::unported!("bot.compose.badnews:_someone_elses_symptoms")
}

/// Python: bot/compose/badnews.py:1132 `reads_for_a_carer(clean, memory=None)`
pub fn reads_for_a_carer(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.badnews:reads_for_a_carer")
}

/// Python: bot/compose/badnews.py:1144 `reads_as_a_carer_body(clean, memory=None)`
pub fn reads_as_a_carer_body(clean: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_a_carer_body")
}

/// Python: bot/compose/badnews.py:1159 `reads_for_a_pet(clean)`
pub fn reads_for_a_pet(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_for_a_pet")
}

/// Python: bot/compose/badnews.py:1168 `reads_as_a_pet_low(clean)` (never called on the trace corpus)
pub fn reads_as_a_pet_low(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_a_pet_low")
}

/// Python: bot/compose/badnews.py:1175 `reads_as_a_pet_condition(clean)` (never called on the trace corpus)
pub fn reads_as_a_pet_condition(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_a_pet_condition")
}

/// Python: bot/compose/badnews.py:1182 `reads_like_a_symptom(clean)`
pub fn reads_like_a_symptom(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_like_a_symptom")
}

/// Python: bot/compose/badnews.py:1190 `_one_edit(left, right)`
pub fn _one_edit(left: &str, right: &str) -> bool {
    crate::unported!("bot.compose.badnews:_one_edit")
}

/// Python: bot/compose/badnews.py:1244 `reads_as_sudden_loss(clean)`
pub fn reads_as_sudden_loss(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_sudden_loss")
}

/// Python: bot/compose/badnews.py:1258 `reads_as_someones_death(clean)`
pub fn reads_as_someones_death(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_someones_death")
}

/// Python: bot/compose/badnews.py:1287 `_subject_before(text, start, said='')` (never called on the trace corpus)
pub fn _subject_before(text: &str, start: i64, said: &str) -> bool {
    crate::unported!("bot.compose.badnews:_subject_before")
}

/// Python: bot/compose/badnews.py:1316 `reads_as_pregnancy_bleeding(clean)`
pub fn reads_as_pregnancy_bleeding(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_pregnancy_bleeding")
}

/// Python: bot/compose/badnews.py:1321 `reads_as_the_speakers_own_plan(clean)` (never called on the trace corpus)
pub fn reads_as_the_speakers_own_plan(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_the_speakers_own_plan")
}

/// Python: bot/compose/badnews.py:1326 `_the_result_came_back_well(clean)`
pub fn _the_result_came_back_well(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:_the_result_came_back_well")
}

/// Python: bot/compose/badnews.py:1339 `reads_as_bad_news(clean)`
pub fn reads_as_bad_news(clean: &str) -> bool {
    crate::unported!("bot.compose.badnews:reads_as_bad_news")
}

