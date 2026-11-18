//! Port of `bot/modules/safety.py`.
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

/// Python: bot/modules/safety.py:117 `_one(verb, who)` (never called on the trace corpus)
pub fn _one(verb: &str, who: &str) -> String {
    crate::unported!("bot.modules.safety:_one")
}

/// Python: bot/modules/safety.py:674 `_person(line)`
pub fn _person(line: &str) -> Option<String> {
    crate::unported!("bot.modules.safety:_person")
}

/// Python: bot/modules/safety.py:748 `_dose_count(word)` (never called on the trace corpus)
pub fn _dose_count(word: &str) -> Value {
    crate::unported!("bot.modules.safety:_dose_count")
}

/// Python: bot/modules/safety.py:759 `_day_dose_reply(text)` (never called on the trace corpus)
pub fn _day_dose_reply(text: &str) -> String {
    crate::unported!("bot.modules.safety:_day_dose_reply")
}

/// Python: bot/modules/safety.py:785 `_stated_dose(line)` (never called on the trace corpus)
pub fn _stated_dose(line: &str) -> Option<String> {
    crate::unported!("bot.modules.safety:_stated_dose")
}

/// Python: bot/modules/safety.py:850 `_fit(reply, line, case=None)`
pub fn _fit(reply: &str, line: &str, case: &str) -> String {
    crate::unported!("bot.modules.safety:_fit")
}

/// Python: bot/modules/safety.py:887 `_letters_apart(word, target)`
pub fn _letters_apart(word: &str, target: &str) -> i64 {
    crate::unported!("bot.modules.safety:_letters_apart")
}

/// Python: bot/modules/safety.py:899 `_near_miss(word, target)`
pub fn _near_miss(word: &str, target: &str) -> bool {
    crate::unported!("bot.modules.safety:_near_miss")
}

/// Python: bot/modules/safety.py:908 `_leet_made_whole(text)`
pub fn _leet_made_whole(text: &str) -> String {
    crate::unported!("bot.modules.safety:_leet_made_whole")
}

/// Python: bot/modules/safety.py:919 `_fold_critical(line)`
pub fn _fold_critical(line: &str) -> String {
    crate::unported!("bot.modules.safety:_fold_critical")
}

/// Python: bot/modules/safety.py:945 `_norm(text)`
pub fn _norm(text: &str) -> String {
    crate::unported!("bot.modules.safety:_norm")
}

/// Python: bot/modules/safety.py:1160 `_ends_it_with_a_method(line)` (never called on the trace corpus)
pub fn _ends_it_with_a_method(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_ends_it_with_a_method")
}

/// Python: bot/modules/safety.py:1538 `_ending_a_thing(line)` (never called on the trace corpus)
pub fn _ending_a_thing(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_ending_a_thing")
}

/// Python: bot/modules/safety.py:1561 `_avoided(line, patterns)` (never called on the trace corpus)
pub fn _avoided(line: &str, patterns: &Value) -> bool {
    crate::unported!("bot.modules.safety:_avoided")
}

/// Python: bot/modules/safety.py:1567 `_deserve(line)`
pub fn _deserve(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_deserve")
}

/// Python: bot/modules/safety.py:1660 `_habit_explained(line)`
pub fn _habit_explained(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_habit_explained")
}

/// Python: bot/modules/safety.py:1680 `_a_pet_hurting_itself(line)`
pub fn _a_pet_hurting_itself(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_a_pet_hurting_itself")
}

/// Python: bot/modules/safety.py:1694 `_a_wish_that_is_theirs(line)`
pub fn _a_wish_that_is_theirs(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_a_wish_that_is_theirs")
}

/// Python: bot/modules/safety.py:1711 `_a_pet_wants_out(line)`
pub fn _a_pet_wants_out(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_a_pet_wants_out")
}

/// Python: bot/modules/safety.py:1730 `_self_harm_intent(line)`
pub fn _self_harm_intent(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_self_harm_intent")
}

/// Python: bot/modules/safety.py:1815 `_someone_elses_despair(line)` (never called on the trace corpus)
pub fn _someone_elses_despair(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_someone_elses_despair")
}

/// Python: bot/modules/safety.py:1917 `_hopeless(line)`
pub fn _hopeless(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_hopeless")
}

/// Python: bot/modules/safety.py:1933 `_matches(patterns, line)`
pub fn _matches(patterns: &Value, line: &str) -> bool {
    crate::unported!("bot.modules.safety:_matches")
}

/// Python: bot/modules/safety.py:1963 `_a_wish_for_their_own_body(line)` (never called on the trace corpus)
pub fn _a_wish_for_their_own_body(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_a_wish_for_their_own_body")
}

/// Python: bot/modules/safety.py:1977 `_idiom(line)`
pub fn _idiom(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_idiom")
}

/// Python: bot/modules/safety.py:2013 `_cut_off_idiom(line)`
pub fn _cut_off_idiom(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_cut_off_idiom")
}

/// Python: bot/modules/safety.py:2032 `_denied_wish(line)`
pub fn _denied_wish(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_denied_wish")
}

/// Python: bot/modules/safety.py:2047 `_blocked(line)` (never called on the trace corpus)
pub fn _blocked(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_blocked")
}

/// Python: bot/modules/safety.py:2067 `_a_days_dose(line)` (never called on the trace corpus)
pub fn _a_days_dose(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_a_days_dose")
}

/// Python: bot/modules/safety.py:2110 `_the_person_is_in_it(line)`
pub fn _the_person_is_in_it(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_the_person_is_in_it")
}

/// Python: bot/modules/safety.py:2120 `_asking_about_the_world(line)`
pub fn _asking_about_the_world(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_asking_about_the_world")
}

/// Python: bot/modules/safety.py:2440 `_nothing_left_clause()` (never called on the trace corpus)
pub fn _nothing_left_clause() -> Value {
    crate::unported!("bot.modules.safety:_nothing_left_clause")
}

/// Python: bot/modules/safety.py:2582 `_a_small_child_did_it(line, memory)` (never called on the trace corpus)
pub fn _a_small_child_did_it(line: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.safety:_a_small_child_did_it")
}

/// Python: bot/modules/safety.py:2650 `_hit_by_a_loss(line)` (never called on the trace corpus)
pub fn _hit_by_a_loss(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_hit_by_a_loss")
}

/// Python: bot/modules/safety.py:2961 `_overdose_of_a_treat(line)` (never called on the trace corpus)
pub fn _overdose_of_a_treat(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_overdose_of_a_treat")
}

/// Python: bot/modules/safety.py:4165 `_glass_test_line(line, before)`
pub fn _glass_test_line(line: &str, before: &str) -> bool {
    crate::unported!("bot.modules.safety:_glass_test_line")
}

/// Python: bot/modules/safety.py:4178 `_said_before(memory, turns=2)`
pub fn _said_before(memory: &Memory, turns: i64) -> String {
    crate::unported!("bot.modules.safety:_said_before")
}

/// Python: bot/modules/safety.py:4192 `_person_before(memory)` (never called on the trace corpus)
pub fn _person_before(memory: &Memory) -> String {
    crate::unported!("bot.modules.safety:_person_before")
}

/// Python: bot/modules/safety.py:4230 `_own_wish(line)`
pub fn _own_wish(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_own_wish")
}

/// Python: bot/modules/safety.py:5104 `_a_reason_it_was_not_meant(text)` (never called on the trace corpus)
pub fn _a_reason_it_was_not_meant(text: &str) -> bool {
    crate::unported!("bot.modules.safety:_a_reason_it_was_not_meant")
}

/// Python: bot/modules/safety.py:5140 `_a_real_injury_beside_the_plate(text)`
pub fn _a_real_injury_beside_the_plate(text: &str) -> bool {
    crate::unported!("bot.modules.safety:_a_real_injury_beside_the_plate")
}

/// Python: bot/modules/safety.py:5162 `_an_ordinary_helping(text)`
pub fn _an_ordinary_helping(text: &str) -> bool {
    crate::unported!("bot.modules.safety:_an_ordinary_helping")
}

/// Python: bot/modules/safety.py:5329 `_rest_of_it(line)`
pub fn _rest_of_it(line: &str) -> String {
    crate::unported!("bot.modules.safety:_rest_of_it")
}

/// Python: bot/modules/safety.py:5348 `_rings(text)` (never called on the trace corpus)
pub fn _rings(text: &str) -> bool {
    crate::unported!("bot.modules.safety:_rings")
}

/// Python: bot/modules/safety.py:5354 `_rest_of_it_rings(line)`
pub fn _rest_of_it_rings(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_rest_of_it_rings")
}

/// Python: bot/modules/safety.py:5555 `_the_laugh(line)`
pub fn _the_laugh(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_the_laugh")
}

/// Python: bot/modules/safety.py:5569 `_an_accident_near_thing(line)`
pub fn _an_accident_near_thing(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_an_accident_near_thing")
}

/// Python: bot/modules/safety.py:5585 `_near_thing_key(line)` (never called on the trace corpus)
pub fn _near_thing_key(line: &str) -> String {
    crate::unported!("bot.modules.safety:_near_thing_key")
}

/// Python: bot/modules/safety.py:5591 `_the_effort_frame(line)`
pub fn _the_effort_frame(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_the_effort_frame")
}

/// Python: bot/modules/safety.py:5741 `_without_object_limbs(line)` (never called on the trace corpus)
pub fn _without_object_limbs(line: &str) -> Value {
    crate::unported!("bot.modules.safety:_without_object_limbs")
}

/// Python: bot/modules/safety.py:5821 `_any_of()` (never called on the trace corpus)
pub fn _any_of(groups: &[Value]) -> Value {
    crate::unported!("bot.modules.safety:_any_of")
}

/// Python: bot/modules/safety.py:5827 `_both(first)` (never called on the trace corpus)
pub fn _both(first: &Value, rest: &[Value]) -> Vec<String> {
    crate::unported!("bot.modules.safety:_both")
}

/// Python: bot/modules/safety.py:6166 `_the_only_fever_is_a_disease_name(line)` (never called on the trace corpus)
pub fn _the_only_fever_is_a_disease_name(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_the_only_fever_is_a_disease_name")
}

/// Python: bot/modules/safety.py:6689 `_for_the_person(reply, line, case)` (never called on the trace corpus)
pub fn _for_the_person(reply: &str, line: &str, case: &Value) -> Value {
    crate::unported!("bot.modules.safety:_for_the_person")
}

/// Python: bot/modules/safety.py:7527 `_fainted_with_the_bang(line)` (never called on the trace corpus)
pub fn _fainted_with_the_bang(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_fainted_with_the_bang")
}

/// Python: bot/modules/safety.py:7564 `_effort_recovery(line)`
pub fn _effort_recovery(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_effort_recovery")
}

/// Python: bot/modules/safety.py:7654 `_hallucinating(line)`
pub fn _hallucinating(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_hallucinating")
}

/// Python: bot/modules/safety.py:7661 `_severe(line)`
pub fn _severe(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_severe")
}

/// Python: bot/modules/safety.py:7921 `_ring_stuck(line)`
pub fn _ring_stuck(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_ring_stuck")
}

/// Python: bot/modules/safety.py:7927 `_friction_blister(line)`
pub fn _friction_blister(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_friction_blister")
}

/// Python: bot/modules/safety.py:7935 `_tooth_out(line)`
pub fn _tooth_out(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_tooth_out")
}

/// Python: bot/modules/safety.py:7965 `_fall_with_sign(line)`
pub fn _fall_with_sign(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_fall_with_sign")
}

/// Python: bot/modules/safety.py:7988 `_out_cold_knockout(line)`
pub fn _out_cold_knockout(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_out_cold_knockout")
}

/// Python: bot/modules/safety.py:7995 `_head_bang(line)`
pub fn _head_bang(line: &str) {
    crate::unported!("bot.modules.safety:_head_bang")
}

/// Python: bot/modules/safety.py:8044 `_head_person(line)` (never called on the trace corpus)
pub fn _head_person(line: &str) -> Value {
    crate::unported!("bot.modules.safety:_head_person")
}

/// Python: bot/modules/safety.py:8060 `_weight_on_a_joint(line, today=False)`
pub fn _weight_on_a_joint(line: &str, today: bool) -> bool {
    crate::unported!("bot.modules.safety:_weight_on_a_joint")
}

/// Python: bot/modules/safety.py:8097 `_grime_on_a_surface(line)`
pub fn _grime_on_a_surface(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_grime_on_a_surface")
}

/// Python: bot/modules/safety.py:8125 `_pet_first_aid(line)` (never called on the trace corpus)
pub fn _pet_first_aid(line: &str) -> Value {
    crate::unported!("bot.modules.safety:_pet_first_aid")
}

/// Python: bot/modules/safety.py:8154 `_the_injury_the_chat_is_on(line, memory)`
pub fn _the_injury_the_chat_is_on(line: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.safety:_the_injury_the_chat_is_on")
}

/// Python: bot/modules/safety.py:8184 `_minor(line, memory=None)`
pub fn _minor(line: &str, memory: &Memory) -> Option<Vec<String>> {
    crate::unported!("bot.modules.safety:_minor")
}

/// Python: bot/modules/safety.py:8202 `_minor_replies(name)` (never called on the trace corpus)
pub fn _minor_replies(name: &str) -> Value {
    crate::unported!("bot.modules.safety:_minor_replies")
}

/// Python: bot/modules/safety.py:8210 `_first_aid(line)`
pub fn _first_aid(line: &str) -> Option<Vec<String>> {
    crate::unported!("bot.modules.safety:_first_aid")
}

/// Python: bot/modules/safety.py:8381 `_last_case(memory)`
pub fn _last_case(memory: &Memory) {
    crate::unported!("bot.modules.safety:_last_case")
}

/// Python: bot/modules/safety.py:8474 `_a_settling_fit(line)` (never called on the trace corpus)
pub fn _a_settling_fit(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_a_settling_fit")
}

/// Python: bot/modules/safety.py:8508 `_last_said_as_written(memory)` (never called on the trace corpus)
pub fn _last_said_as_written(memory: &Memory) -> Value {
    crate::unported!("bot.modules.safety:_last_said_as_written")
}

/// Python: bot/modules/safety.py:8518 `_resay(raw, line, memory)` (never called on the trace corpus)
pub fn _resay(raw: &str, line: &str, memory: &mut Memory) -> Value {
    crate::unported!("bot.modules.safety:_resay")
}

/// Python: bot/modules/safety.py:8548 `_name_the_person(reply, line, memory)` (never called on the trace corpus)
pub fn _name_the_person(reply: &str, line: &str, memory: &Memory) -> Value {
    crate::unported!("bot.modules.safety:_name_the_person")
}

/// Python: bot/modules/safety.py:8562 `_true_newborn(memory)` (never called on the trace corpus)
pub fn _true_newborn(memory: &Memory) -> bool {
    crate::unported!("bot.modules.safety:_true_newborn")
}

/// Python: bot/modules/safety.py:8584 `_tiny_baby(memory)`
pub fn _tiny_baby(memory: &Memory) -> bool {
    crate::unported!("bot.modules.safety:_tiny_baby")
}

/// Python: bot/modules/safety.py:8606 `_last_said(memory)`
pub fn _last_said(memory: &Memory) -> String {
    crate::unported!("bot.modules.safety:_last_said")
}

/// Python: bot/modules/safety.py:8672 `_follow_up(line, memory)`
pub fn _follow_up(line: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.safety:_follow_up")
}

/// Python: bot/modules/safety.py:8882 `_note(memory, case)`
pub fn _note(memory: &mut Memory, case: &str) {
    crate::unported!("bot.modules.safety:_note")
}

/// Python: bot/modules/safety.py:8902 `_sugar_is_the_shopping(line)` (never called on the trace corpus)
pub fn _sugar_is_the_shopping(line: &str) -> Value {
    crate::unported!("bot.modules.safety:_sugar_is_the_shopping")
}

/// Python: bot/modules/safety.py:8917 `_hypo_reply(line, memory)`
pub fn _hypo_reply(line: &str, memory: &Memory) {
    crate::unported!("bot.modules.safety:_hypo_reply")
}

/// Python: bot/modules/safety.py:8948 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    crate::unported!("bot.modules.safety:_words")
}

/// Python: bot/modules/safety.py:8953 `_a_repair_the_reader_made(memory)`
pub fn _a_repair_the_reader_made(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.modules.safety:_a_repair_the_reader_made")
}

/// Python: bot/modules/safety.py:8970 `_repaired(text, memory)`
pub fn _repaired(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.safety:_repaired")
}

/// Python: bot/modules/safety.py:9037 `_withdrawal(line)`
pub fn _withdrawal(line: &str) {
    crate::unported!("bot.modules.safety:_withdrawal")
}

/// Python: bot/modules/safety.py:9112 `_says_something_is_burning(line)` (never called on the trace corpus)
pub fn _says_something_is_burning(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_says_something_is_burning")
}

/// Python: bot/modules/safety.py:9126 `_fire_words_in(line)` (never called on the trace corpus)
pub fn _fire_words_in(line: &str) -> Value {
    crate::unported!("bot.modules.safety:_fire_words_in")
}

/// Python: bot/modules/safety.py:9143 `_a_fire_that_did_not_happen(line)` (never called on the trace corpus)
pub fn _a_fire_that_did_not_happen(line: &str) -> bool {
    crate::unported!("bot.modules.safety:_a_fire_that_did_not_happen")
}

/// Python: bot/modules/safety.py:9166 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.safety:respond")
}

