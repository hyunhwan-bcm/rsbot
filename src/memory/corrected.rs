//! Port of `bot/memory/corrected.py`.
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

/// Python: bot/memory/corrected.py:17 `_number_belongs_to_that_age(memory)` (never called on the trace corpus)
pub fn _number_belongs_to_that_age(memory: &Memory) -> bool {
    crate::unported!("bot.memory.corrected:_number_belongs_to_that_age")
}

/// Python: bot/memory/corrected.py:43 `_a_name_for_a_person(value)` (never called on the trace corpus)
pub fn _a_name_for_a_person(value: &Value) -> bool {
    crate::unported!("bot.memory.corrected:_a_name_for_a_person")
}

/// Python: bot/memory/corrected.py:64 `_a_new_name_was_said(target, text)` (never called on the trace corpus)
pub fn _a_new_name_was_said(target: &Value, text: &str) -> bool {
    crate::unported!("bot.memory.corrected:_a_new_name_was_said")
}

/// Python: bot/memory/corrected.py:90 `_a_word_of_a_name(word)` (never called on the trace corpus)
pub fn _a_word_of_a_name(word: &str) -> bool {
    crate::unported!("bot.memory.corrected:_a_word_of_a_name")
}

/// Python: bot/memory/corrected.py:101 `_denies_and_names(text)` (never called on the trace corpus)
pub fn _denies_and_names(text: &str) -> bool {
    crate::unported!("bot.memory.corrected:_denies_and_names")
}

/// Python: bot/memory/corrected.py:114 `_correct_other(memory, target, text='')` (never called on the trace corpus)
pub fn _correct_other(memory: &mut Memory, target: &Value, text: &str) -> Value {
    crate::unported!("bot.memory.corrected:_correct_other")
}

/// Python: bot/memory/corrected.py:186 `_make_corrects(target, memory)` (never called on the trace corpus)
pub fn _make_corrects(target: &Value, memory: &Memory) -> bool {
    crate::unported!("bot.memory.corrected:_make_corrects")
}

/// Python: bot/memory/corrected.py:201 `_the_unit(text)` (never called on the trace corpus)
pub fn _the_unit(text: &str) -> Value {
    crate::unported!("bot.memory.corrected:_the_unit")
}

/// Python: bot/memory/corrected.py:208 `_stated_families(before)` (never called on the trace corpus)
pub fn _stated_families(before: &str) -> Value {
    crate::unported!("bot.memory.corrected:_stated_families")
}

/// Python: bot/memory/corrected.py:224 `_measured_the_same(value, before)` (never called on the trace corpus)
pub fn _measured_the_same(value: &Value, before: &str) -> bool {
    crate::unported!("bot.memory.corrected:_measured_the_same")
}

/// Python: bot/memory/corrected.py:247 `_without_the_naming_verb(value)` (never called on the trace corpus)
pub fn _without_the_naming_verb(value: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_without_the_naming_verb")
}

/// Python: bot/memory/corrected.py:264 `_the_plan_on_the_table(memory)` (never called on the trace corpus)
pub fn _the_plan_on_the_table(memory: &Memory) -> Value {
    crate::unported!("bot.memory.corrected:_the_plan_on_the_table")
}

/// Python: bot/memory/corrected.py:283 `_the_plan_before(memory, plan_after)` (never called on the trace corpus)
pub fn _the_plan_before(memory: &Memory, plan_after: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_the_plan_before")
}

/// Python: bot/memory/corrected.py:301 `_what_took_its_place(before, after, value)` (never called on the trace corpus)
pub fn _what_took_its_place(before: &str, after: &str, value: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_what_took_its_place")
}

/// Python: bot/memory/corrected.py:338 `_a_day_written_the_way_the_store_writes_it(fact)` (never called on the trace corpus)
pub fn _a_day_written_the_way_the_store_writes_it(fact: &str) -> Value {
    crate::unported!("bot.memory.corrected:_a_day_written_the_way_the_store_writes_it")
}

/// Python: bot/memory/corrected.py:346 `_the_person_the_plan_is_about(memory, correction, before=None)` (never called on the trace corpus)
pub fn _the_person_the_plan_is_about(memory: &mut Memory, correction: &Value, before: &str) -> Value {
    crate::unported!("bot.memory.corrected:_the_person_the_plan_is_about")
}

/// Python: bot/memory/corrected.py:397 `_a_correction_is_not_a_plan(memory, text)` (never called on the trace corpus)
pub fn _a_correction_is_not_a_plan(memory: &mut Memory, text: &str) {
    crate::unported!("bot.memory.corrected:_a_correction_is_not_a_plan")
}

/// Python: bot/memory/corrected.py:421 `_a_value_already_the_plans(memory, target, text='')` (never called on the trace corpus)
pub fn _a_value_already_the_plans(memory: &mut Memory, target: &Value, text: &str) -> bool {
    crate::unported!("bot.memory.corrected:_a_value_already_the_plans")
}

/// Python: bot/memory/corrected.py:450 `_apply_correction(memory, correction, text='')` (never called on the trace corpus)
pub fn _apply_correction(memory: &mut Memory, correction: &Value, text: &str) {
    crate::unported!("bot.memory.corrected:_apply_correction")
}

/// Python: bot/memory/corrected.py:629 `_stated_plan(memory)` (never called on the trace corpus)
pub fn _stated_plan(memory: &Memory) -> Value {
    crate::unported!("bot.memory.corrected:_stated_plan")
}

/// Python: bot/memory/corrected.py:650 `_an_amount_on_the_measure(plan, target)` (never called on the trace corpus)
pub fn _an_amount_on_the_measure(plan: &str, target: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_an_amount_on_the_measure")
}

/// Python: bot/memory/corrected.py:684 `_correct_plan(memory, target, wrong, text='')` (never called on the trace corpus)
pub fn _correct_plan(memory: &mut Memory, target: &Value, wrong: &Value, text: &str) -> Value {
    crate::unported!("bot.memory.corrected:_correct_plan")
}

/// Python: bot/memory/corrected.py:767 `_number_in(value)` (never called on the trace corpus)
pub fn _number_in(value: &Value) -> String {
    crate::unported!("bot.memory.corrected:_number_in")
}

/// Python: bot/memory/corrected.py:772 `_correct_a_persons_name(memory, target, text='', wrong='')` (never called on the trace corpus)
pub fn _correct_a_persons_name(memory: &mut Memory, target: &Value, text: &str, wrong: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_correct_a_persons_name")
}

/// Python: bot/memory/corrected.py:830 `_a_spelling_of(new, old)` (never called on the trace corpus)
pub fn _a_spelling_of(new: &Value, old: &Value) -> bool {
    crate::unported!("bot.memory.corrected:_a_spelling_of")
}

/// Python: bot/memory/corrected.py:846 `_letters_apart(one, other)` (never called on the trace corpus)
pub fn _letters_apart(one: &str, other: &str) -> Value {
    crate::unported!("bot.memory.corrected:_letters_apart")
}

/// Python: bot/memory/corrected.py:859 `_correct_clock(memory, target, text='', wrong='')` (never called on the trace corpus)
pub fn _correct_clock(memory: &mut Memory, target: &Value, text: &str, wrong: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_correct_clock")
}

/// Python: bot/memory/corrected.py:901 `_the_topic_with_the_new_clock(text, plan_now)` (never called on the trace corpus)
pub fn _the_topic_with_the_new_clock(text: &str, plan_now: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_the_topic_with_the_new_clock")
}

/// Python: bot/memory/corrected.py:933 `_a_bare_hour(value)` (never called on the trace corpus)
pub fn _a_bare_hour(value: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_a_bare_hour")
}

/// Python: bot/memory/corrected.py:940 `_names_a_half_of_the_day(clock)` (never called on the trace corpus)
pub fn _names_a_half_of_the_day(clock: &Value) -> bool {
    crate::unported!("bot.memory.corrected:_names_a_half_of_the_day")
}

/// Python: bot/memory/corrected.py:944 `_clock_key(clock)` (never called on the trace corpus)
pub fn _clock_key(clock: &Value) -> String {
    crate::unported!("bot.memory.corrected:_clock_key")
}

/// Python: bot/memory/corrected.py:948 `_correct_number(memory, target, text='')` (never called on the trace corpus)
pub fn _correct_number(memory: &mut Memory, target: &Value, text: &str) -> Value {
    crate::unported!("bot.memory.corrected:_correct_number")
}

/// Python: bot/memory/corrected.py:1031 `_count_said(text)` (never called on the trace corpus)
pub fn _count_said(text: &str) -> i64 {
    crate::unported!("bot.memory.corrected:_count_said")
}

/// Python: bot/memory/corrected.py:1041 `_a_count(count, animal)` (never called on the trace corpus)
pub fn _a_count(count: i64, animal: &Value) -> String {
    crate::unported!("bot.memory.corrected:_a_count")
}

/// Python: bot/memory/corrected.py:1050 `_correct_count(memory, target, text='')` (never called on the trace corpus)
pub fn _correct_count(memory: &mut Memory, target: &Value, text: &str) -> Value {
    crate::unported!("bot.memory.corrected:_correct_count")
}

/// Python: bot/memory/corrected.py:1104 `_correct_the_count_on_the_table(memory, target, text='')` (never called on the trace corpus)
pub fn _correct_the_count_on_the_table(memory: &mut Memory, target: &Value, text: &str) -> Value {
    crate::unported!("bot.memory.corrected:_correct_the_count_on_the_table")
}

/// Python: bot/memory/corrected.py:1145 `_a_fact_of_somebody_else(memory, plan)` (never called on the trace corpus)
pub fn _a_fact_of_somebody_else(memory: &Memory, plan: &str) -> bool {
    crate::unported!("bot.memory.corrected:_a_fact_of_somebody_else")
}

/// Python: bot/memory/corrected.py:1165 `_the_plans_a_journey_moves(memory, plans, text)` (never called on the trace corpus)
pub fn _the_plans_a_journey_moves(memory: &Memory, plans: &Value, text: &str) -> Value {
    crate::unported!("bot.memory.corrected:_the_plans_a_journey_moves")
}

/// Python: bot/memory/corrected.py:1176 `_correct_when(memory, target, text='')` (never called on the trace corpus)
pub fn _correct_when(memory: &mut Memory, target: &Value, text: &str) -> Value {
    crate::unported!("bot.memory.corrected:_correct_when")
}

/// Python: bot/memory/corrected.py:1229 `_date_said(low)` (never called on the trace corpus)
pub fn _date_said(low: &str) -> Value {
    crate::unported!("bot.memory.corrected:_date_said")
}

/// Python: bot/memory/corrected.py:1245 `_a_day_in_the_kind(old_day, day)` (never called on the trace corpus)
pub fn _a_day_in_the_kind(old_day: &Value, day: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_a_day_in_the_kind")
}

/// Python: bot/memory/corrected.py:1263 `_date_word_case(value)` (never called on the trace corpus)
pub fn _date_word_case(value: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_date_word_case")
}

/// Python: bot/memory/corrected.py:1268 `_date_in_plan(plan)` (never called on the trace corpus)
pub fn _date_in_plan(plan: &str) -> Value {
    crate::unported!("bot.memory.corrected:_date_in_plan")
}

/// Python: bot/memory/corrected.py:1276 `_the_weekday_in(plan)` (never called on the trace corpus)
pub fn _the_weekday_in(plan: &str) -> Value {
    crate::unported!("bot.memory.corrected:_the_weekday_in")
}

/// Python: bot/memory/corrected.py:1280 `_names_a_date(value)` (never called on the trace corpus)
pub fn _names_a_date(value: &Value) -> bool {
    crate::unported!("bot.memory.corrected:_names_a_date")
}

/// Python: bot/memory/corrected.py:1288 `_a_relative_month(plan, day, month)` (never called on the trace corpus)
pub fn _a_relative_month(plan: &str, day: &Value, month: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_a_relative_month")
}

/// Python: bot/memory/corrected.py:1310 `_a_relative_line(memory, plan, was, said)` (never called on the trace corpus)
pub fn _a_relative_line(memory: &mut Memory, plan: &str, was: &str, said: &str) -> Option<String> {
    crate::unported!("bot.memory.corrected:_a_relative_line")
}

/// Python: bot/memory/corrected.py:1321 `_plan_saying(memory, day, month)` (never called on the trace corpus)
pub fn _plan_saying(memory: &Memory, day: &Value, month: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_plan_saying")
}

/// Python: bot/memory/corrected.py:1340 `_correct_date(memory, target, text='', wrong='')` (never called on the trace corpus)
pub fn _correct_date(memory: &mut Memory, target: &Value, text: &str, wrong: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_correct_date")
}

/// Python: bot/memory/corrected.py:1451 `_a_name_the_store_keeps(memory, taken_back, new)` (never called on the trace corpus)
pub fn _a_name_the_store_keeps(memory: &mut Memory, taken_back: &Value, new: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_a_name_the_store_keeps")
}

/// Python: bot/memory/corrected.py:1478 `_stated_last(memory)` (never called on the trace corpus)
pub fn _stated_last(memory: &Memory) -> Value {
    crate::unported!("bot.memory.corrected:_stated_last")
}

/// Python: bot/memory/corrected.py:1547 `_plan_told_line(memory, old, new, text='')` (never called on the trace corpus)
pub fn _plan_told_line(memory: &mut Memory, old: &Value, new: &Value, text: &str) -> Value {
    crate::unported!("bot.memory.corrected:_plan_told_line")
}

/// Python: bot/memory/corrected.py:1562 `_date_word(phrase)` (never called on the trace corpus)
pub fn _date_word(phrase: &str) -> Value {
    crate::unported!("bot.memory.corrected:_date_word")
}

/// Python: bot/memory/corrected.py:1567 `_kind_of_value(target, told)` (never called on the trace corpus)
pub fn _kind_of_value(target: &Value, told: &Value) -> Option<String> {
    crate::unported!("bot.memory.corrected:_kind_of_value")
}

/// Python: bot/memory/corrected.py:1590 `_swap_in(told, target, wrong=None)` (never called on the trace corpus)
pub fn _swap_in(told: &Value, target: &Value, wrong: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_swap_in")
}

/// Python: bot/memory/corrected.py:1665 `_plan_the_line_named(plans, told)` (never called on the trace corpus)
pub fn _plan_the_line_named(plans: &Value, told: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_plan_the_line_named")
}

/// Python: bot/memory/corrected.py:1682 `_carries(value, word)` (never called on the trace corpus)
pub fn _carries(value: &Value, word: &str) -> bool {
    crate::unported!("bot.memory.corrected:_carries")
}

/// Python: bot/memory/corrected.py:1688 `_fact_naming(memory, wrong)` (never called on the trace corpus)
pub fn _fact_naming(memory: &Memory, wrong: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_fact_naming")
}

/// Python: bot/memory/corrected.py:1710 `_a_birthday_on_the_last_line(memory)` (never called on the trace corpus)
pub fn _a_birthday_on_the_last_line(memory: &Memory) -> bool {
    crate::unported!("bot.memory.corrected:_a_birthday_on_the_last_line")
}

/// Python: bot/memory/corrected.py:1729 `_correct_birthday(memory, target, text='')` (never called on the trace corpus)
pub fn _correct_birthday(memory: &mut Memory, target: &Value, text: &str) -> Value {
    crate::unported!("bot.memory.corrected:_correct_birthday")
}

/// Python: bot/memory/corrected.py:1763 `_a_birthday_for_somebody(memory)` (never called on the trace corpus)
pub fn _a_birthday_for_somebody(memory: &mut Memory) -> Value {
    crate::unported!("bot.memory.corrected:_a_birthday_for_somebody")
}

/// Python: bot/memory/corrected.py:1795 `_the_line_is_about_that_birthday(text, held, target)` (never called on the trace corpus)
pub fn _the_line_is_about_that_birthday(text: &str, held: &Value, target: &Value) -> bool {
    crate::unported!("bot.memory.corrected:_the_line_is_about_that_birthday")
}

/// Python: bot/memory/corrected.py:1814 `_correct_their_birthday(memory, target, wrong, text='')` (never called on the trace corpus)
pub fn _correct_their_birthday(memory: &mut Memory, target: &Value, wrong: &Value, text: &str) -> Value {
    crate::unported!("bot.memory.corrected:_correct_their_birthday")
}

/// Python: bot/memory/corrected.py:1868 `_correct_fact(memory, target, wrong, text='')` (never called on the trace corpus)
pub fn _correct_fact(memory: &mut Memory, target: &Value, wrong: &Value, text: &str) -> Value {
    crate::unported!("bot.memory.corrected:_correct_fact")
}

/// Python: bot/memory/corrected.py:2032 `_keep_unit(value, was)` (never called on the trace corpus)
pub fn _keep_unit(value: &Value, was: &str) -> Value {
    crate::unported!("bot.memory.corrected:_keep_unit")
}

/// Python: bot/memory/corrected.py:2041 `_drop_wrong(known, key, wrong, target)` (never called on the trace corpus)
pub fn _drop_wrong(known: &Value, key: &str, wrong: &Value, target: &Value) {
    crate::unported!("bot.memory.corrected:_drop_wrong")
}

/// Python: bot/memory/corrected.py:2062 `_birthday_words(date)` (never called on the trace corpus)
pub fn _birthday_words(date: &Value) -> Value {
    crate::unported!("bot.memory.corrected:_birthday_words")
}

/// Python: bot/memory/corrected.py:2082 `_many(count, animal)` (never called on the trace corpus)
pub fn _many(count: i64, animal: &Value) -> String {
    crate::unported!("bot.memory.corrected:_many")
}

/// Python: bot/memory/corrected.py:2098 `_asks(line)`
pub fn _asks(line: &str) -> bool {
    crate::unported!("bot.memory.corrected:_asks")
}

