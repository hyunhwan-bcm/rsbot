//! Port of `bot/compose/loss.py`.
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

/// Python: bot/compose/loss.py:150 `_only_a_new_subject(line)`
pub fn _only_a_new_subject(line: &str) -> bool {
    crate::unported!("bot.compose.loss:_only_a_new_subject")
}

/// Python: bot/compose/loss.py:159 `_only_the_proceedings(clean, words)`
pub fn _only_the_proceedings(clean: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_only_the_proceedings")
}

/// Python: bot/compose/loss.py:505 `_killed_in_a_story(clean)` (never called on the trace corpus)
pub fn _killed_in_a_story(clean: &str) -> bool {
    crate::unported!("bot.compose.loss:_killed_in_a_story")
}

/// Python: bot/compose/loss.py:619 `_dead_is_not_a_death(words, index)` (never called on the trace corpus)
pub fn _dead_is_not_a_death(words: &[String], index: i64) -> bool {
    crate::unported!("bot.compose.loss:_dead_is_not_a_death")
}

/// Python: bot/compose/loss.py:905 `is_dying_now(text)`
pub fn is_dying_now(text: &str) -> bool {
    crate::unported!("bot.compose.loss:is_dying_now")
}

/// Python: bot/compose/loss.py:949 `dying_in_the_chat(memory, turns=6)`
pub fn dying_in_the_chat(memory: &Memory, turns: i64) -> bool {
    crate::unported!("bot.compose.loss:dying_in_the_chat")
}

/// Python: bot/compose/loss.py:966 `_a_pet_of_ours(line, words=None)` (never called on the trace corpus)
pub fn _a_pet_of_ours(line: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_a_pet_of_ours")
}

/// Python: bot/compose/loss.py:974 `_the_end_is_near(tight, words=None)`
pub fn _the_end_is_near(tight: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_the_end_is_near")
}

/// Python: bot/compose/loss.py:1057 `_the_family_is_told(tight)`
pub fn _the_family_is_told(tight: &str) -> bool {
    crate::unported!("bot.compose.loss:_the_family_is_told")
}

/// Python: bot/compose/loss.py:1085 `_the_vet_said_it(tight, words)` (never called on the trace corpus)
pub fn _the_vet_said_it(tight: &Value, words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_the_vet_said_it")
}

/// Python: bot/compose/loss.py:1105 `_the_give_up_is_about_somebody(tight, words, third_person_only=False)` (never called on the trace corpus)
pub fn _the_give_up_is_about_somebody(tight: &Value, words: &[String], third_person_only: &Value) -> bool {
    crate::unported!("bot.compose.loss:_the_give_up_is_about_somebody")
}

/// Python: bot/compose/loss.py:1154 `_the_give_up_without_a_care_word(tight, who)`
pub fn _the_give_up_without_a_care_word(tight: &str, who: bool) -> bool {
    crate::unported!("bot.compose.loss:_the_give_up_without_a_care_word")
}

/// Python: bot/compose/loss.py:1493 `_the_bill_was_the_last_turn(memory)`
pub fn _the_bill_was_the_last_turn(memory: &Memory) -> bool {
    crate::unported!("bot.compose.loss:_the_bill_was_the_last_turn")
}

/// Python: bot/compose/loss.py:1511 `_loss_is_the_thread(memory)`
pub fn _loss_is_the_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.loss:_loss_is_the_thread")
}

/// Python: bot/compose/loss.py:1520 `arrangement_cost_pool(line, tight=None, memory=None)`
pub fn arrangement_cost_pool(line: &str, tight: &str, memory: &Memory) {
    crate::unported!("bot.compose.loss:arrangement_cost_pool")
}

/// Python: bot/compose/loss.py:1703 `says_the_last_drug(tight)`
pub fn says_the_last_drug(tight: &str) -> bool {
    crate::unported!("bot.compose.loss:says_the_last_drug")
}

/// Python: bot/compose/loss.py:1842 `asks_about_the_remains(line, tight=None)` (never called on the trace corpus)
pub fn asks_about_the_remains(line: &str, tight: &Value) -> bool {
    crate::unported!("bot.compose.loss:asks_about_the_remains")
}

/// Python: bot/compose/loss.py:1869 `a_pet_died(line)` (never called on the trace corpus)
pub fn a_pet_died(line: &str) -> bool {
    crate::unported!("bot.compose.loss:a_pet_died")
}

/// Python: bot/compose/loss.py:1919 `asks_the_administration(line)`
pub fn asks_the_administration(line: &str) -> bool {
    crate::unported!("bot.compose.loss:asks_the_administration")
}

/// Python: bot/compose/loss.py:1956 `administration_keys(text)` (never called on the trace corpus)
pub fn administration_keys(text: &str) -> Value {
    crate::unported!("bot.compose.loss:administration_keys")
}

/// Python: bot/compose/loss.py:1975 `is_about_the_paperwork(reply, line)` (never called on the trace corpus)
pub fn is_about_the_paperwork(reply: &str, line: &str) -> bool {
    crate::unported!("bot.compose.loss:is_about_the_paperwork")
}

/// Python: bot/compose/loss.py:2070 `_a_thing_of_its_own_to_get_through(line)` (never called on the trace corpus)
pub fn _a_thing_of_its_own_to_get_through(line: &str) -> bool {
    crate::unported!("bot.compose.loss:_a_thing_of_its_own_to_get_through")
}

/// Python: bot/compose/loss.py:2087 `asks_for_a_decision(line)` (never called on the trace corpus)
pub fn asks_for_a_decision(line: &str) -> bool {
    crate::unported!("bot.compose.loss:asks_for_a_decision")
}

/// Python: bot/compose/loss.py:2115 `feeling_about_the_body(line)` (never called on the trace corpus)
pub fn feeling_about_the_body(line: &str) -> bool {
    crate::unported!("bot.compose.loss:feeling_about_the_body")
}

/// Python: bot/compose/loss.py:2120 `names_a_feeling(line)` (never called on the trace corpus)
pub fn names_a_feeling(line: &str) -> bool {
    crate::unported!("bot.compose.loss:names_a_feeling")
}

/// Python: bot/compose/loss.py:2134 `acknowledges(reply)`
pub fn acknowledges(reply: &str) -> bool {
    crate::unported!("bot.compose.loss:acknowledges")
}

/// Python: bot/compose/loss.py:2158 `_when_asked(tight)` (never called on the trace corpus)
pub fn _when_asked(tight: &Value) -> bool {
    crate::unported!("bot.compose.loss:_when_asked")
}

/// Python: bot/compose/loss.py:2163 `a_part_of_the_day_asked(tight)`
pub fn a_part_of_the_day_asked(tight: &str) -> bool {
    crate::unported!("bot.compose.loss:a_part_of_the_day_asked")
}

/// Python: bot/compose/loss.py:2192 `_asks_about_the_feeling(tight)` (never called on the trace corpus)
pub fn _asks_about_the_feeling(tight: &Value) -> bool {
    crate::unported!("bot.compose.loss:_asks_about_the_feeling")
}

/// Python: bot/compose/loss.py:2326 `_feeling(tight)`
pub fn _feeling(tight: &str) -> String {
    crate::unported!("bot.compose.loss:_feeling")
}

/// Python: bot/compose/loss.py:2451 `_how_long_since(tight, words)`
pub fn _how_long_since(tight: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_how_long_since")
}

/// Python: bot/compose/loss.py:2551 `_missing_someone(text)`
pub fn _missing_someone(text: &str) -> bool {
    crate::unported!("bot.compose.loss:_missing_someone")
}

/// Python: bot/compose/loss.py:2602 `_a_question_about_the_feeling(line, words)` (never called on the trace corpus)
pub fn _a_question_about_the_feeling(line: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_a_question_about_the_feeling")
}

/// Python: bot/compose/loss.py:2804 `_an_ask_in_the_line(line)`
pub fn _an_ask_in_the_line(line: &str) -> bool {
    crate::unported!("bot.compose.loss:_an_ask_in_the_line")
}

/// Python: bot/compose/loss.py:2839 `_still_here(line, words)`
pub fn _still_here(line: &str, words: &[String]) {
    crate::unported!("bot.compose.loss:_still_here")
}

/// Python: bot/compose/loss.py:2866 `_a_habit_in(line)`
pub fn _a_habit_in(line: &str) -> bool {
    crate::unported!("bot.compose.loss:_a_habit_in")
}

/// Python: bot/compose/loss.py:3077 `_telling_a_child(tight)`
pub fn _telling_a_child(tight: &str) -> bool {
    crate::unported!("bot.compose.loss:_telling_a_child")
}

/// Python: bot/compose/loss.py:3097 `marks_a_day(text, known=False)`
pub fn marks_a_day(text: &str, known: bool) -> bool {
    crate::unported!("bot.compose.loss:marks_a_day")
}

/// Python: bot/compose/loss.py:3150 `asks_what_to_do(text)` (never called on the trace corpus)
pub fn asks_what_to_do(text: &str) -> bool {
    crate::unported!("bot.compose.loss:asks_what_to_do")
}

/// Python: bot/compose/loss.py:3167 `_plain(text)`
pub fn _plain(text: &str) -> Vec<String> {
    crate::unported!("bot.compose.loss:_plain")
}

/// Python: bot/compose/loss.py:3173 `_about_them(line)`
pub fn _about_them(line: &str) -> bool {
    crate::unported!("bot.compose.loss:_about_them")
}

/// Python: bot/compose/loss.py:3182 `_about_family(words)`
pub fn _about_family(words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_about_family")
}

/// Python: bot/compose/loss.py:3188 `_own_state(words)`
pub fn _own_state(words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_own_state")
}

/// Python: bot/compose/loss.py:3216 `_state_describes_them(words)`
pub fn _state_describes_them(words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_state_describes_them")
}

/// Python: bot/compose/loss.py:3253 `_the_body_is_theirs(tight, words)` (never called on the trace corpus)
pub fn _the_body_is_theirs(tight: &Value, words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_the_body_is_theirs")
}

/// Python: bot/compose/loss.py:3281 `_a_body_with_its_own_answer(tight, words)` (never called on the trace corpus)
pub fn _a_body_with_its_own_answer(tight: &Value, words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_a_body_with_its_own_answer")
}

/// Python: bot/compose/loss.py:3295 `_names_a_subject(topics)` (never called on the trace corpus)
pub fn _names_a_subject(topics: &Value) -> bool {
    crate::unported!("bot.compose.loss:_names_a_subject")
}

/// Python: bot/compose/loss.py:3306 `_own_subject(topics)`
pub fn _own_subject(topics: Option<&[String]>) -> bool {
    crate::unported!("bot.compose.loss:_own_subject")
}

/// Python: bot/compose/loss.py:3338 `_hands_over_a_detail(tight)` (never called on the trace corpus)
pub fn _hands_over_a_detail(tight: &Value) -> bool {
    crate::unported!("bot.compose.loss:_hands_over_a_detail")
}

/// Python: bot/compose/loss.py:3351 `_thread_pool(memory)`
pub fn _thread_pool(memory: &Memory) {
    crate::unported!("bot.compose.loss:_thread_pool")
}

/// Python: bot/compose/loss.py:3395 `_spoken(text)`
pub fn _spoken(text: &str) -> String {
    crate::unported!("bot.compose.loss:_spoken")
}

/// Python: bot/compose/loss.py:3411 `_tight(text)`
pub fn _tight(text: &str) -> String {
    crate::unported!("bot.compose.loss:_tight")
}

/// Python: bot/compose/loss.py:3417 `_their_thing_after(clean, start)` (never called on the trace corpus)
pub fn _their_thing_after(clean: &str, start: i64) -> bool {
    crate::unported!("bot.compose.loss:_their_thing_after")
}

/// Python: bot/compose/loss.py:3427 `_another_person(words, clean)`
pub fn _another_person(words: &[String], clean: &str) -> bool {
    crate::unported!("bot.compose.loss:_another_person")
}

/// Python: bot/compose/loss.py:3439 `_a_thing_of_theirs(words)`
pub fn _a_thing_of_theirs(words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_a_thing_of_theirs")
}

/// Python: bot/compose/loss.py:3461 `_a_role_phrase(clean)`
pub fn _a_role_phrase(clean: &str) -> bool {
    crate::unported!("bot.compose.loss:_a_role_phrase")
}

/// Python: bot/compose/loss.py:3471 `_has_person(words)`
pub fn _has_person(words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_has_person")
}

/// Python: bot/compose/loss.py:3494 `_object_died(words)` (never called on the trace corpus)
pub fn _object_died(words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_object_died")
}

/// Python: bot/compose/loss.py:3589 `_where_i_put_it(said)`
pub fn _where_i_put_it(said: &str) -> bool {
    crate::unported!("bot.compose.loss:_where_i_put_it")
}

/// Python: bot/compose/loss.py:3623 `locked_out(text)`
pub fn locked_out(text: &str) -> bool {
    crate::unported!("bot.compose.loss:locked_out")
}

/// Python: bot/compose/loss.py:3636 `locked_out_of_the_car(text)` (never called on the trace corpus)
pub fn locked_out_of_the_car(text: &str) -> bool {
    crate::unported!("bot.compose.loss:locked_out_of_the_car")
}

/// Python: bot/compose/loss.py:3641 `_readable(text)`
pub fn _readable(text: &str) -> String {
    crate::unported!("bot.compose.loss:_readable")
}

/// Python: bot/compose/loss.py:3656 `lost_thing_to_find(text)`
pub fn lost_thing_to_find(text: &str) -> bool {
    crate::unported!("bot.compose.loss:lost_thing_to_find")
}

/// Python: bot/compose/loss.py:3677 `keeps_losing_thing(text)`
pub fn keeps_losing_thing(text: &str) -> bool {
    crate::unported!("bot.compose.loss:keeps_losing_thing")
}

/// Python: bot/compose/loss.py:3686 `found_what_was_lost(text)`
pub fn found_what_was_lost(text: &str) -> bool {
    crate::unported!("bot.compose.loss:found_what_was_lost")
}

/// Python: bot/compose/loss.py:3695 `lost_in_a_place(text)`
pub fn lost_in_a_place(text: &str) -> bool {
    crate::unported!("bot.compose.loss:lost_in_a_place")
}

/// Python: bot/compose/loss.py:3718 `_shed_after(clean, start)` (never called on the trace corpus)
pub fn _shed_after(clean: &str, start: i64) -> bool {
    crate::unported!("bot.compose.loss:_shed_after")
}

/// Python: bot/compose/loss.py:3735 `_feared_not_done(clean, start)` (never called on the trace corpus)
pub fn _feared_not_done(clean: &str, start: i64) -> bool {
    crate::unported!("bot.compose.loss:_feared_not_done")
}

/// Python: bot/compose/loss.py:3748 `_lost_in_a_place_after(clean, start)` (never called on the trace corpus)
pub fn _lost_in_a_place_after(clean: &str, start: i64) -> bool {
    crate::unported!("bot.compose.loss:_lost_in_a_place_after")
}

/// Python: bot/compose/loss.py:3758 `_can_die(clean)`
pub fn _can_die(clean: &str) -> Vec<String> {
    crate::unported!("bot.compose.loss:_can_die")
}

/// Python: bot/compose/loss.py:3768 `lost_a_thing(text)`
pub fn lost_a_thing(text: &str) -> bool {
    crate::unported!("bot.compose.loss:lost_a_thing")
}

/// Python: bot/compose/loss.py:3794 `words_of(text)`
pub fn words_of(text: &str) -> Vec<String> {
    crate::unported!("bot.compose.loss:words_of")
}

/// Python: bot/compose/loss.py:3806 `_brought_in_prey(clean, words)`
pub fn _brought_in_prey(clean: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_brought_in_prey")
}

/// Python: bot/compose/loss.py:3820 `_a_pronoun_is_what_died(words, clean)`
pub fn _a_pronoun_is_what_died(words: &[String], clean: &str) -> bool {
    crate::unported!("bot.compose.loss:_a_pronoun_is_what_died")
}

/// Python: bot/compose/loss.py:3832 `a_thing_stopped(text, words=None)` (never called on the trace corpus)
pub fn a_thing_stopped(text: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:a_thing_stopped")
}

/// Python: bot/compose/loss.py:3855 `reads_as_loss(text, words=None, a_pet=False)`
pub fn reads_as_loss(text: &str, words: Option<&[String]>, a_pet: bool) -> bool {
    crate::unported!("bot.compose.loss:reads_as_loss")
}

/// Python: bot/compose/loss.py:4027 `_accident_death(clean)`
pub fn _accident_death(clean: &str) -> bool {
    crate::unported!("bot.compose.loss:_accident_death")
}

/// Python: bot/compose/loss.py:4050 `_owned_deathable(clean)` (never called on the trace corpus)
pub fn _owned_deathable(clean: &str) -> Value {
    crate::unported!("bot.compose.loss:_owned_deathable")
}

/// Python: bot/compose/loss.py:4063 `_passed_on(clean, person)`
pub fn _passed_on(clean: &str, person: bool) -> bool {
    crate::unported!("bot.compose.loss:_passed_on")
}

/// Python: bot/compose/loss.py:4086 `_passed_alone(clean, person)`
pub fn _passed_alone(clean: &str, person: bool) -> bool {
    crate::unported!("bot.compose.loss:_passed_alone")
}

/// Python: bot/compose/loss.py:4129 `_death_noun(clean, words, person)`
pub fn _death_noun(clean: &str, words: &[String], person: bool) -> bool {
    crate::unported!("bot.compose.loss:_death_noun")
}

/// Python: bot/compose/loss.py:4162 `_unnamed_person(words, clean)`
pub fn _unnamed_person(words: &[String], clean: &str) -> bool {
    crate::unported!("bot.compose.loss:_unnamed_person")
}

/// Python: bot/compose/loss.py:4194 `_buried_person(clean, words, person=False)`
pub fn _buried_person(clean: &str, words: &[String], person: bool) -> bool {
    crate::unported!("bot.compose.loss:_buried_person")
}

/// Python: bot/compose/loss.py:4229 `_put_down(clean, words)`
pub fn _put_down(clean: &str, words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_put_down")
}

/// Python: bot/compose/loss.py:4243 `_names_a_stored_pet(text, memory)`
pub fn _names_a_stored_pet(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.loss:_names_a_stored_pet")
}

/// Python: bot/compose/loss.py:4258 `_has_pet(words)`
pub fn _has_pet(words: &[String]) -> bool {
    crate::unported!("bot.compose.loss:_has_pet")
}

/// Python: bot/compose/loss.py:4262 `_a_person_dying(tight)` (never called on the trace corpus)
pub fn _a_person_dying(tight: &Value) -> bool {
    crate::unported!("bot.compose.loss:_a_person_dying")
}

/// Python: bot/compose/loss.py:4277 `_a_hard_time_that_is_theirs(line)`
pub fn _a_hard_time_that_is_theirs(line: &str) -> bool {
    crate::unported!("bot.compose.loss:_a_hard_time_that_is_theirs")
}

/// Python: bot/compose/loss.py:4307 `whose(clean)`
pub fn whose(clean: &str) -> String {
    crate::unported!("bot.compose.loss:whose")
}

/// Python: bot/compose/loss.py:4364 `one_of(animal)` (never called on the trace corpus)
pub fn one_of(animal: &Value) -> Value {
    crate::unported!("bot.compose.loss:one_of")
}

/// Python: bot/compose/loss.py:4377 `the_animal(line)`
pub fn the_animal(line: &str) -> String {
    crate::unported!("bot.compose.loss:the_animal")
}

/// Python: bot/compose/loss.py:4394 `named_animal(replies, line)` (never called on the trace corpus)
pub fn named_animal(replies: &Value, line: &str) -> Value {
    crate::unported!("bot.compose.loss:named_animal")
}

/// Python: bot/compose/loss.py:4404 `a_relative_pet(line)` (never called on the trace corpus)
pub fn a_relative_pet(line: &str) -> bool {
    crate::unported!("bot.compose.loss:a_relative_pet")
}

/// Python: bot/compose/loss.py:4410 `a_strangers_animal(line)` (never called on the trace corpus)
pub fn a_strangers_animal(line: &str) -> bool {
    crate::unported!("bot.compose.loss:a_strangers_animal")
}

/// Python: bot/compose/loss.py:4438 `_a_swap_of_the_question_before(text, memory)`
pub fn _a_swap_of_the_question_before(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.loss:_a_swap_of_the_question_before")
}

/// Python: bot/compose/loss.py:4467 `_still_ahead(memory)` (never called on the trace corpus)
pub fn _still_ahead(memory: &Memory) -> Value {
    crate::unported!("bot.compose.loss:_still_ahead")
}

/// Python: bot/compose/loss.py:4507 `_less_what_was_said(pool, turns)` (never called on the trace corpus)
pub fn _less_what_was_said(pool: &Value, turns: &[Turn]) -> Value {
    crate::unported!("bot.compose.loss:_less_what_was_said")
}

/// Python: bot/compose/loss.py:4591 `_an_animal_in_the_chat(memory, turns=6)` (never called on the trace corpus)
pub fn _an_animal_in_the_chat(memory: &Memory, turns: &[Turn]) -> bool {
    crate::unported!("bot.compose.loss:_an_animal_in_the_chat")
}

/// Python: bot/compose/loss.py:4608 `_keeping_going(tight, words, memory)`
pub fn _keeping_going(tight: &str, words: &[String], memory: &Memory) {
    crate::unported!("bot.compose.loss:_keeping_going")
}

/// Python: bot/compose/loss.py:4629 `_first_days(memory)` (never called on the trace corpus)
pub fn _first_days(memory: &Memory) -> Value {
    crate::unported!("bot.compose.loss:_first_days")
}

/// Python: bot/compose/loss.py:4652 `_the_visit_ask(clean)`
pub fn _the_visit_ask(clean: &str) -> bool {
    crate::unported!("bot.compose.loss:_the_visit_ask")
}

/// Python: bot/compose/loss.py:4662 `_the_animal_of(text, memory)` (never called on the trace corpus)
pub fn _the_animal_of(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.loss:_the_animal_of")
}

/// Python: bot/compose/loss.py:4673 `_grief_replies(line, memory)` (never called on the trace corpus)
pub fn _grief_replies(line: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.loss:_grief_replies")
}

/// Python: bot/compose/loss.py:4693 `_a_colour_is_worn(line)`
pub fn _a_colour_is_worn(line: &str) -> bool {
    crate::unported!("bot.compose.loss:_a_colour_is_worn")
}

/// Python: bot/compose/loss.py:4702 `reply_for(text, memory, clean=None, topics=None, mood=None)`
pub fn reply_for(text: &str, memory: &Memory, clean: &str, topics: &[String], mood: Option<&str>) {
    crate::unported!("bot.compose.loss:reply_for")
}

/// Python: bot/compose/loss.py:5156 `asks_anything(line, tight=None)`
pub fn asks_anything(line: &str, tight: &str) -> bool {
    crate::unported!("bot.compose.loss:asks_anything")
}

/// Python: bot/compose/loss.py:5208 `_a_gift_for_someone_in_care(tight)`
pub fn _a_gift_for_someone_in_care(tight: &str) -> bool {
    crate::unported!("bot.compose.loss:_a_gift_for_someone_in_care")
}

/// Python: bot/compose/loss.py:5214 `is_practical_ask(line, tight=None)`
pub fn is_practical_ask(line: &str, tight: &str) -> bool {
    crate::unported!("bot.compose.loss:is_practical_ask")
}

/// Python: bot/compose/loss.py:5301 `_an_ending_in_the_chat(memory)`
pub fn _an_ending_in_the_chat(memory: &Memory) -> bool {
    crate::unported!("bot.compose.loss:_an_ending_in_the_chat")
}

/// Python: bot/compose/loss.py:5322 `continues(memory)`
pub fn continues(memory: &Memory) -> bool {
    crate::unported!("bot.compose.loss:continues")
}

