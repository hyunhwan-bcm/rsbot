//! Port of `bot/compose/why.py`.
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

/// Python: bot/compose/why.py:94 `_named_as_a_sentence(last, reason)` (never called on the trace corpus)
pub fn _named_as_a_sentence(last: &str, reason: &Value) -> bool {
    crate::unported!("bot.compose.why:_named_as_a_sentence")
}

/// Python: bot/compose/why.py:111 `_reads_a_fault(last)` (never called on the trace corpus)
pub fn _reads_a_fault(last: &str) -> bool {
    crate::unported!("bot.compose.why:_reads_a_fault")
}

/// Python: bot/compose/why.py:149 `_about_someone_elses_decision(memory)` (never called on the trace corpus)
pub fn _about_someone_elses_decision(memory: &Memory) -> bool {
    crate::unported!("bot.compose.why:_about_someone_elses_decision")
}

/// Python: bot/compose/why.py:193 `_the_suggestion_was_theirs(memory)` (never called on the trace corpus)
pub fn _the_suggestion_was_theirs(memory: &Memory) -> bool {
    crate::unported!("bot.compose.why:_the_suggestion_was_theirs")
}

/// Python: bot/compose/why.py:460 `_in_frame(frame, clause)` (never called on the trace corpus)
pub fn _in_frame(frame: &Value, clause: &str) -> String {
    crate::unported!("bot.compose.why:_in_frame")
}

/// Python: bot/compose/why.py:499 `is_a_reason_line(text)`
pub fn is_a_reason_line(text: &str) -> bool {
    crate::unported!("bot.compose.why:is_a_reason_line")
}

/// Python: bot/compose/why.py:569 `_said_the_reason_cannot_be_seen(memory)` (never called on the trace corpus)
pub fn _said_the_reason_cannot_be_seen(memory: &Memory) -> bool {
    crate::unported!("bot.compose.why:_said_the_reason_cannot_be_seen")
}

/// Python: bot/compose/why.py:596 `_is_bare_why(clean)`
pub fn _is_bare_why(clean: &str) -> bool {
    crate::unported!("bot.compose.why:_is_bare_why")
}

/// Python: bot/compose/why.py:601 `_last_turn(memory)` (never called on the trace corpus)
pub fn _last_turn(memory: &Memory) -> Value {
    crate::unported!("bot.compose.why:_last_turn")
}

/// Python: bot/compose/why.py:664 `_answered_nothing(said)` (never called on the trace corpus)
pub fn _answered_nothing(said: &str) -> bool {
    crate::unported!("bot.compose.why:_answered_nothing")
}

/// Python: bot/compose/why.py:693 `_gave_a_method(last)` (never called on the trace corpus)
pub fn _gave_a_method(last: &str) -> bool {
    crate::unported!("bot.compose.why:_gave_a_method")
}

/// Python: bot/compose/why.py:705 `_a_cause_named_outright(last)` (never called on the trace corpus)
pub fn _a_cause_named_outright(last: &str) -> Value {
    crate::unported!("bot.compose.why:_a_cause_named_outright")
}

/// Python: bot/compose/why.py:729 `_reason_in(last, strict=False)` (never called on the trace corpus)
pub fn _reason_in(last: &str, strict: &Value) -> Value {
    crate::unported!("bot.compose.why:_reason_in")
}

/// Python: bot/compose/why.py:754 `_without_examples(said)` (never called on the trace corpus)
pub fn _without_examples(said: &str) -> Value {
    crate::unported!("bot.compose.why:_without_examples")
}

/// Python: bot/compose/why.py:759 `_inside_a_quotation(said, clause)` (never called on the trace corpus)
pub fn _inside_a_quotation(said: &str, clause: &str) -> bool {
    crate::unported!("bot.compose.why:_inside_a_quotation")
}

/// Python: bot/compose/why.py:770 `_a_reason_in(said, strict=False)` (never called on the trace corpus)
pub fn _a_reason_in(said: &str, strict: &Value) -> Value {
    crate::unported!("bot.compose.why:_a_reason_in")
}

/// Python: bot/compose/why.py:946 `_is_the_answer_itself(said, sentence)` (never called on the trace corpus)
pub fn _is_the_answer_itself(said: &str, sentence: &str) -> bool {
    crate::unported!("bot.compose.why:_is_the_answer_itself")
}

/// Python: bot/compose/why.py:968 `_words(text)` (never called on the trace corpus)
pub fn _words(text: &str) -> Value {
    crate::unported!("bot.compose.why:_words")
}

/// Python: bot/compose/why.py:973 `_carries_the_fix(sentence, clause)` (never called on the trace corpus)
pub fn _carries_the_fix(sentence: &str, clause: &str) -> bool {
    crate::unported!("bot.compose.why:_carries_the_fix")
}

/// Python: bot/compose/why.py:982 `_trigger_in(part)` (never called on the trace corpus)
pub fn _trigger_in(part: &str) -> Value {
    crate::unported!("bot.compose.why:_trigger_in")
}

/// Python: bot/compose/why.py:1013 `_cause_clause(part, number, instruction_ok)` (never called on the trace corpus)
pub fn _cause_clause(part: &str, number: &Value, instruction_ok: &Value) -> Value {
    crate::unported!("bot.compose.why:_cause_clause")
}

/// Python: bot/compose/why.py:1094 `_the_class_it_belongs_to(said)` (never called on the trace corpus)
pub fn _the_class_it_belongs_to(said: &str) -> String {
    crate::unported!("bot.compose.why:_the_class_it_belongs_to")
}

/// Python: bot/compose/why.py:1124 `_named_by_definition(said)` (never called on the trace corpus)
pub fn _named_by_definition(said: &str) -> String {
    crate::unported!("bot.compose.why:_named_by_definition")
}

/// Python: bot/compose/why.py:1155 `_cut_long(clause)` (never called on the trace corpus)
pub fn _cut_long(clause: &str) -> Value {
    crate::unported!("bot.compose.why:_cut_long")
}

/// Python: bot/compose/why.py:1204 `_an_answer_to_act_on(last)` (never called on the trace corpus)
pub fn _an_answer_to_act_on(last: &str) -> bool {
    crate::unported!("bot.compose.why:_an_answer_to_act_on")
}

/// Python: bot/compose/why.py:1318 `_first_sentence(said)` (never called on the trace corpus)
pub fn _first_sentence(said: &str) -> String {
    crate::unported!("bot.compose.why:_first_sentence")
}

/// Python: bot/compose/why.py:1329 `_a_shorter_way(said)` (never called on the trace corpus)
pub fn _a_shorter_way(said: &str) -> Value {
    crate::unported!("bot.compose.why:_a_shorter_way")
}

/// Python: bot/compose/why.py:1389 `_past_a_lead_in(said)` (never called on the trace corpus)
pub fn _past_a_lead_in(said: &str) -> Value {
    crate::unported!("bot.compose.why:_past_a_lead_in")
}

/// Python: bot/compose/why.py:1405 `_end_a_sentence(text)` (never called on the trace corpus)
pub fn _end_a_sentence(text: &str) -> Value {
    crate::unported!("bot.compose.why:_end_a_sentence")
}

/// Python: bot/compose/why.py:1413 `_a_clause_of(sentence)` (never called on the trace corpus)
pub fn _a_clause_of(sentence: &str) -> Value {
    crate::unported!("bot.compose.why:_a_clause_of")
}

/// Python: bot/compose/why.py:1441 `_the_doing_of(sentence)` (never called on the trace corpus)
pub fn _the_doing_of(sentence: &str) -> Value {
    crate::unported!("bot.compose.why:_the_doing_of")
}

/// Python: bot/compose/why.py:1497 `_opens_a_doing(half)` (never called on the trace corpus)
pub fn _opens_a_doing(half: &Value) -> bool {
    crate::unported!("bot.compose.why:_opens_a_doing")
}

/// Python: bot/compose/why.py:1526 `_says_how_it_works(half)` (never called on the trace corpus)
pub fn _says_how_it_works(half: &Value) -> bool {
    crate::unported!("bot.compose.why:_says_how_it_works")
}

/// Python: bot/compose/why.py:1540 `_announces_the_list(half)` (never called on the trace corpus)
pub fn _announces_the_list(half: &Value) -> bool {
    crate::unported!("bot.compose.why:_announces_the_list")
}

/// Python: bot/compose/why.py:1553 `_needs_its_subject(after)` (never called on the trace corpus)
pub fn _needs_its_subject(after: &str) -> bool {
    crate::unported!("bot.compose.why:_needs_its_subject")
}

/// Python: bot/compose/why.py:1581 `_carries_a_verb(half)` (never called on the trace corpus)
pub fn _carries_a_verb(half: &Value) -> bool {
    crate::unported!("bot.compose.why:_carries_a_verb")
}

/// Python: bot/compose/why.py:1613 `_leans_back(half)` (never called on the trace corpus)
pub fn _leans_back(half: &Value) -> bool {
    crate::unported!("bot.compose.why:_leans_back")
}

/// Python: bot/compose/why.py:1637 `_the_reason(after)` (never called on the trace corpus)
pub fn _the_reason(after: &str) -> bool {
    crate::unported!("bot.compose.why:_the_reason")
}

/// Python: bot/compose/why.py:1652 `_opens_on_a_figure(after)` (never called on the trace corpus)
pub fn _opens_on_a_figure(after: &str) -> bool {
    crate::unported!("bot.compose.why:_opens_on_a_figure")
}

/// Python: bot/compose/why.py:1668 `_names_no_particular(before)` (never called on the trace corpus)
pub fn _names_no_particular(before: &str) -> bool {
    crate::unported!("bot.compose.why:_names_no_particular")
}

/// Python: bot/compose/why.py:1673 `_the_half_that_does_it(before, after)` (never called on the trace corpus)
pub fn _the_half_that_does_it(before: &str, after: &str) -> String {
    crate::unported!("bot.compose.why:_the_half_that_does_it")
}

/// Python: bot/compose/why.py:1769 `_points_at_what_follows(cut)` (never called on the trace corpus)
pub fn _points_at_what_follows(cut: &Value) -> bool {
    crate::unported!("bot.compose.why:_points_at_what_follows")
}

/// Python: bot/compose/why.py:1777 `_brackets_closed(text)` (never called on the trace corpus)
pub fn _brackets_closed(text: &str) -> bool {
    crate::unported!("bot.compose.why:_brackets_closed")
}

/// Python: bot/compose/why.py:1783 `_the_route_is_in_it(said)` (never called on the trace corpus)
pub fn _the_route_is_in_it(said: &str) -> bool {
    crate::unported!("bot.compose.why:_the_route_is_in_it")
}

/// Python: bot/compose/why.py:1788 `said_again(clean, memory)`
pub fn said_again(clean: &str, memory: &Memory) {
    crate::unported!("bot.compose.why:said_again")
}

/// Python: bot/compose/why.py:1809 `is_the_answer_shorter(reply, last)`
pub fn is_the_answer_shorter(reply: &str, last: &str) -> bool {
    crate::unported!("bot.compose.why:is_the_answer_shorter")
}

/// Python: bot/compose/why.py:1842 `_said_above(memory)` (never called on the trace corpus)
pub fn _said_above(memory: &Memory) -> Value {
    crate::unported!("bot.compose.why:_said_above")
}

/// Python: bot/compose/why.py:1853 `_unframed(reply)` (never called on the trace corpus)
pub fn _unframed(reply: &str) -> Value {
    crate::unported!("bot.compose.why:_unframed")
}

/// Python: bot/compose/why.py:1872 `_not_shortened(said)` (never called on the trace corpus)
pub fn _not_shortened(said: &str) -> Value {
    crate::unported!("bot.compose.why:_not_shortened")
}

/// Python: bot/compose/why.py:1912 `takes_as_it_comes(text, memory)`
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.why:takes_as_it_comes")
}

/// Python: bot/compose/why.py:2030 `asks_about_the_last_answer(clean)`
pub fn asks_about_the_last_answer(clean: &str) -> bool {
    crate::unported!("bot.compose.why:asks_about_the_last_answer")
}

/// Python: bot/compose/why.py:2035 `respond_to_a_saying_ask(clean, memory, understood=None, body=False)`
pub fn respond_to_a_saying_ask(clean: &str, memory: &Memory, understood: &Understood, body: bool) {
    crate::unported!("bot.compose.why:respond_to_a_saying_ask")
}

/// Python: bot/compose/why.py:2094 `_the_turn_above(memory)` (never called on the trace corpus)
pub fn _the_turn_above(memory: &Memory) -> Value {
    crate::unported!("bot.compose.why:_the_turn_above")
}

/// Python: bot/compose/why.py:2103 `respond(clean, memory, understood=None, body=False)`
pub fn respond(clean: &str, memory: &Memory, understood: &Understood, body: bool) {
    crate::unported!("bot.compose.why:respond")
}

