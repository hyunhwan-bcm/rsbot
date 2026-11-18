//! Port of `bot/modules`: every module, in the order `bot/runtime.py` runs them
//! (highest PRIORITY first, then by file name). Generated from PyBot's own module list.
#![allow(clippy::all)]

use crate::runtime::ModuleDef;

pub mod a_cake_between_its_layers;
pub mod a_check_with_you;
pub mod a_count_of_packets;
pub mod a_count_said;
pub mod a_day_somebody_else_keeps;
pub mod a_debt_said_flat;
pub mod a_fact_asked_again;
pub mod a_figure_beside_the_state;
pub mod a_figure_of_a_dish;
pub mod a_first_time;
pub mod a_habit_is_raised;
pub mod a_job_that_is_not_a_diy_job;
pub mod a_letter_about_money;
pub mod a_limit_over_the_answer;
pub mod a_measure_asked_again;
pub mod a_number_the_bot_knows;
pub mod a_plan_the_thread_holds;
pub mod a_question_about_somebody;
pub mod a_return_brings_back_the_advice;
pub mod a_shock_at_a_bill;
pub mod a_thing_being_weighed;
pub mod a_thing_wants_doing;
pub mod a_will_it_work;
pub mod aboutachild;
pub mod after_a_lead_in;
pub mod already;
pub mod an_age_ahead;
pub mod answers;
pub mod arbiter;
pub mod attribute_said_again;
pub mod bulky;
pub mod callback_about;
pub mod carers_turn;
pub mod chasing_a_body;
pub mod chat;
pub mod chat_replies;
pub mod colour_on_the_body;
pub mod confusing;
pub mod detail_question;
pub mod done_ask;
pub mod doubt;
pub mod dreams;
pub mod familyask;
pub mod favourites;
pub mod figure_asked_again;
pub mod first_line_open;
pub mod greeting;
pub mod grief;
pub mod group_ages;
pub mod hardpast;
pub mod household;
pub mod how_long_a_stay;
pub mod intent;
pub mod invitation;
pub mod judge_ask;
pub mod keeping;
pub mod kid_tally;
pub mod live_source;
pub mod live_times;
pub mod loss;
pub mod memory_store;
pub mod memory_tables;
pub(crate) mod mod4;
pub mod my_work;
pub mod name_taken_back;
pub mod near_me;
pub mod night;
pub mod no_advice;
pub mod nudge;
pub mod on_record;
pub mod onsubject;
pub mod pace;
pub mod particles;
pub mod recap;
pub mod receiving_end;
pub mod record_due;
pub mod ruled_out;
pub mod safety;
pub mod safety_cards;
pub mod safety_phrases;
pub mod safety_tables;
pub mod said_back;
pub mod said_fact_back;
pub mod saving_followup;
pub mod since_it_began;
pub mod steps;
pub mod taking_asked_again;
pub mod talkto;
pub mod the_detail_the_thread_adds;
pub mod the_kind_one_line_up;
pub mod the_month_they_have;
pub mod the_shape_of_the_answer;
pub mod the_trouble_it_has;
pub mod the_trouble_named;
pub mod the_wait_about_a_body;
pub mod their_life;
pub mod their_pastime;
pub mod their_taste;
pub mod theirperson;
pub mod thing_i_did_with;
pub mod trip_date_said_again;
pub mod understand;
pub mod urgent;
pub mod ward;
pub mod what_decides_it;
pub mod what_they_said;
pub mod where_it_answers;
pub mod who_to_see;
pub mod words;
pub mod years_in;

/// Every module, highest PRIORITY first, then by name.
pub fn registry() -> &'static [ModuleDef] {
    static REGISTRY: &[ModuleDef] = &[
        ModuleDef { name: "understand", priority: 10000.0, observe: Some(understand::observe), respond: None, after: None },
        ModuleDef { name: "intent", priority: 9500.0, observe: Some(intent::observe), respond: None, after: None },
        ModuleDef { name: "a_count_of_packets", priority: 9000.0, observe: None, respond: Some(a_count_of_packets::respond), after: None },
        ModuleDef { name: "safety", priority: 9000.0, observe: None, respond: Some(safety::respond), after: None },
        ModuleDef { name: "the_month_they_have", priority: 8000.0, observe: None, respond: Some(the_month_they_have::respond), after: None },
        ModuleDef { name: "since_it_began", priority: 7500.0, observe: None, respond: Some(since_it_began::respond), after: None },
        ModuleDef { name: "keeping", priority: 7400.0, observe: None, respond: Some(keeping::respond), after: None },
        ModuleDef { name: "a_question_about_somebody", priority: 7300.0, observe: None, respond: Some(a_question_about_somebody::respond), after: None },
        ModuleDef { name: "an_age_ahead", priority: 7150.0, observe: None, respond: Some(an_age_ahead::respond), after: None },
        ModuleDef { name: "steps", priority: 7100.0, observe: None, respond: Some(steps::respond), after: None },
        ModuleDef { name: "what_they_said", priority: 7060.0, observe: None, respond: Some(what_they_said::respond), after: None },
        ModuleDef { name: "a_fact_asked_again", priority: 7050.0, observe: None, respond: Some(a_fact_asked_again::respond), after: None },
        ModuleDef { name: "a_return_brings_back_the_advice", priority: 7050.0, observe: None, respond: Some(a_return_brings_back_the_advice::respond), after: None },
        ModuleDef { name: "group_ages", priority: 7050.0, observe: Some(group_ages::observe), respond: Some(group_ages::respond), after: None },
        ModuleDef { name: "household", priority: 7050.0, observe: Some(household::observe), respond: Some(household::respond), after: None },
        ModuleDef { name: "kid_tally", priority: 7050.0, observe: None, respond: Some(kid_tally::respond), after: None },
        ModuleDef { name: "said_fact_back", priority: 7050.0, observe: Some(said_fact_back::observe), respond: Some(said_fact_back::respond), after: None },
        ModuleDef { name: "live_times", priority: 7001.0, observe: None, respond: Some(live_times::respond), after: None },
        ModuleDef { name: "a_count_said", priority: 7000.0, observe: Some(a_count_said::observe), respond: Some(a_count_said::respond), after: None },
        ModuleDef { name: "favourites", priority: 7000.0, observe: Some(favourites::observe), respond: Some(favourites::respond), after: None },
        ModuleDef { name: "memory_store", priority: 7000.0, observe: Some(memory_store::observe), respond: Some(memory_store::respond), after: Some(memory_store::after) },
        ModuleDef { name: "memory_tables", priority: 7000.0, observe: None, respond: None, after: None },
        ModuleDef { name: "recap", priority: 7000.0, observe: None, respond: Some(recap::respond), after: None },
        ModuleDef { name: "attribute_said_again", priority: 6900.0, observe: None, respond: Some(attribute_said_again::respond), after: None },
        ModuleDef { name: "done_ask", priority: 6900.0, observe: Some(done_ask::observe), respond: None, after: None },
        ModuleDef { name: "thing_i_did_with", priority: 6900.0, observe: None, respond: Some(thing_i_did_with::respond), after: None },
        ModuleDef { name: "trip_date_said_again", priority: 6900.0, observe: Some(trip_date_said_again::observe), respond: None, after: None },
        ModuleDef { name: "name_taken_back", priority: 6600.0, observe: Some(name_taken_back::observe), respond: Some(name_taken_back::respond), after: None },
        ModuleDef { name: "dreams", priority: 3000.0, observe: None, respond: Some(dreams::respond), after: None },
        ModuleDef { name: "grief", priority: 2100.0, observe: Some(grief::observe), respond: Some(grief::respond), after: None },
        ModuleDef { name: "loss", priority: 2000.0, observe: None, respond: Some(loss::respond), after: None },
        ModuleDef { name: "who_to_see", priority: 1500.0, observe: None, respond: Some(who_to_see::respond), after: None },
        ModuleDef { name: "pace", priority: 1300.0, observe: None, respond: Some(pace::respond), after: None },
        ModuleDef { name: "a_cake_between_its_layers", priority: 1200.0, observe: None, respond: Some(a_cake_between_its_layers::respond), after: None },
        ModuleDef { name: "words", priority: 1200.0, observe: None, respond: Some(words::respond), after: None },
        ModuleDef { name: "my_work", priority: 1150.0, observe: None, respond: Some(my_work::respond), after: None },
        ModuleDef { name: "years_in", priority: 1150.0, observe: None, respond: Some(years_in::respond), after: None },
        ModuleDef { name: "aboutachild", priority: 1100.0, observe: None, respond: Some(aboutachild::respond), after: None },
        ModuleDef { name: "familyask", priority: 1100.0, observe: None, respond: Some(familyask::respond), after: None },
        ModuleDef { name: "night", priority: 1100.0, observe: None, respond: Some(night::respond), after: None },
        ModuleDef { name: "saving_followup", priority: 1100.0, observe: None, respond: Some(saving_followup::respond), after: None },
        ModuleDef { name: "how_long_a_stay", priority: 1050.0, observe: None, respond: Some(how_long_a_stay::respond), after: None },
        ModuleDef { name: "no_advice", priority: 1050.0, observe: None, respond: Some(no_advice::respond), after: None },
        ModuleDef { name: "a_figure_beside_the_state", priority: 1000.0, observe: None, respond: Some(a_figure_beside_the_state::respond), after: None },
        ModuleDef { name: "a_measure_asked_again", priority: 1000.0, observe: None, respond: Some(a_measure_asked_again::respond), after: None },
        ModuleDef { name: "a_shock_at_a_bill", priority: 1000.0, observe: None, respond: Some(a_shock_at_a_bill::respond), after: None },
        ModuleDef { name: "a_will_it_work", priority: 1000.0, observe: None, respond: Some(a_will_it_work::respond), after: None },
        ModuleDef { name: "figure_asked_again", priority: 1000.0, observe: None, respond: Some(figure_asked_again::respond), after: None },
        ModuleDef { name: "taking_asked_again", priority: 1000.0, observe: None, respond: Some(taking_asked_again::respond), after: None },
        ModuleDef { name: "a_limit_over_the_answer", priority: 500.0, observe: Some(a_limit_over_the_answer::observe), respond: Some(a_limit_over_the_answer::respond), after: None },
        ModuleDef { name: "already", priority: 400.0, observe: None, respond: Some(already::respond), after: None },
        ModuleDef { name: "doubt", priority: 400.0, observe: None, respond: Some(doubt::respond), after: None },
        ModuleDef { name: "colour_on_the_body", priority: 350.0, observe: None, respond: Some(colour_on_the_body::respond), after: None },
        ModuleDef { name: "said_back", priority: 350.0, observe: None, respond: Some(said_back::respond), after: None },
        ModuleDef { name: "after_a_lead_in", priority: 320.0, observe: None, respond: Some(after_a_lead_in::respond), after: None },
        ModuleDef { name: "chat", priority: 300.0, observe: Some(chat::observe), respond: Some(chat::respond), after: Some(chat::after) },
        ModuleDef { name: "their_pastime", priority: 300.0, observe: None, respond: Some(their_pastime::respond), after: None },
        ModuleDef { name: "callback_about", priority: 260.0, observe: None, respond: Some(callback_about::respond), after: None },
        ModuleDef { name: "theirperson", priority: 250.0, observe: None, respond: Some(theirperson::respond), after: None },
        ModuleDef { name: "their_life", priority: 100.0, observe: None, respond: Some(their_life::respond), after: None },
        ModuleDef { name: "answers", priority: 0.0, observe: Some(answers::observe), respond: Some(answers::respond), after: None },
        ModuleDef { name: "arbiter", priority: 0.0, observe: None, respond: None, after: None },
        ModuleDef { name: "chat_replies", priority: 0.0, observe: None, respond: None, after: None },
        ModuleDef { name: "on_record", priority: 0.0, observe: None, respond: None, after: None },
        ModuleDef { name: "particles", priority: 0.0, observe: None, respond: None, after: None },
        ModuleDef { name: "safety_cards", priority: 0.0, observe: None, respond: None, after: None },
        ModuleDef { name: "safety_phrases", priority: 0.0, observe: None, respond: None, after: None },
        ModuleDef { name: "safety_tables", priority: 0.0, observe: None, respond: None, after: None },
        ModuleDef { name: "ruled_out", priority: -450.0, observe: None, respond: Some(ruled_out::respond), after: None },
        ModuleDef { name: "a_debt_said_flat", priority: -500.0, observe: Some(a_debt_said_flat::observe), respond: Some(a_debt_said_flat::respond), after: None },
        ModuleDef { name: "invitation", priority: -500.0, observe: None, respond: Some(invitation::respond), after: None },
        ModuleDef { name: "judge_ask", priority: -500.0, observe: None, respond: Some(judge_ask::respond), after: None },
        ModuleDef { name: "record_due", priority: -500.0, observe: None, respond: Some(record_due::respond), after: None },
        ModuleDef { name: "talkto", priority: -500.0, observe: None, respond: Some(talkto::respond), after: None },
        ModuleDef { name: "a_letter_about_money", priority: -600.0, observe: None, respond: Some(a_letter_about_money::respond), after: None },
        ModuleDef { name: "confusing", priority: -900.0, observe: None, respond: Some(confusing::respond), after: None },
        ModuleDef { name: "urgent", priority: -900.0, observe: None, respond: Some(urgent::respond), after: None },
        ModuleDef { name: "nudge", priority: -950.0, observe: None, respond: Some(nudge::respond), after: None },
        ModuleDef { name: "their_taste", priority: -960.0, observe: None, respond: Some(their_taste::respond), after: None },
        ModuleDef { name: "hardpast", priority: -980.0, observe: None, respond: Some(hardpast::respond), after: None },
        ModuleDef { name: "receiving_end", priority: -980.0, observe: None, respond: Some(receiving_end::respond), after: None },
        ModuleDef { name: "ward", priority: -985.0, observe: None, respond: Some(ward::respond), after: None },
        ModuleDef { name: "near_me", priority: -995.0, observe: None, respond: Some(near_me::respond), after: None },
        ModuleDef { name: "a_first_time", priority: -996.0, observe: None, respond: Some(a_first_time::respond), after: None },
        ModuleDef { name: "a_figure_of_a_dish", priority: -997.0, observe: None, respond: Some(a_figure_of_a_dish::respond), after: None },
        ModuleDef { name: "chasing_a_body", priority: -997.0, observe: None, respond: Some(chasing_a_body::respond), after: None },
        ModuleDef { name: "detail_question", priority: -997.0, observe: None, respond: Some(detail_question::respond), after: None },
        ModuleDef { name: "the_trouble_named", priority: -997.0, observe: None, respond: Some(the_trouble_named::respond), after: None },
        ModuleDef { name: "carers_turn", priority: -998.0, observe: None, respond: Some(carers_turn::respond), after: None },
        ModuleDef { name: "live_source", priority: -998.0, observe: None, respond: Some(live_source::respond), after: None },
        ModuleDef { name: "onsubject", priority: -998.0, observe: None, respond: Some(onsubject::respond), after: None },
        ModuleDef { name: "what_decides_it", priority: -998.0, observe: None, respond: Some(what_decides_it::respond), after: None },
        ModuleDef { name: "a_check_with_you", priority: -999.0, observe: None, respond: Some(a_check_with_you::respond), after: None },
        ModuleDef { name: "a_day_somebody_else_keeps", priority: -999.0, observe: None, respond: Some(a_day_somebody_else_keeps::respond), after: None },
        ModuleDef { name: "a_habit_is_raised", priority: -999.0, observe: None, respond: Some(a_habit_is_raised::respond), after: None },
        ModuleDef { name: "a_job_that_is_not_a_diy_job", priority: -999.0, observe: None, respond: Some(a_job_that_is_not_a_diy_job::respond), after: None },
        ModuleDef { name: "a_number_the_bot_knows", priority: -999.0, observe: None, respond: Some(a_number_the_bot_knows::respond), after: None },
        ModuleDef { name: "a_plan_the_thread_holds", priority: -999.0, observe: None, respond: Some(a_plan_the_thread_holds::respond), after: None },
        ModuleDef { name: "a_thing_being_weighed", priority: -999.0, observe: None, respond: Some(a_thing_being_weighed::respond), after: None },
        ModuleDef { name: "a_thing_wants_doing", priority: -999.0, observe: None, respond: Some(a_thing_wants_doing::respond), after: None },
        ModuleDef { name: "bulky", priority: -999.0, observe: None, respond: Some(bulky::respond), after: None },
        ModuleDef { name: "the_detail_the_thread_adds", priority: -999.0, observe: None, respond: Some(the_detail_the_thread_adds::respond), after: None },
        ModuleDef { name: "the_kind_one_line_up", priority: -999.0, observe: None, respond: Some(the_kind_one_line_up::respond), after: None },
        ModuleDef { name: "the_trouble_it_has", priority: -999.0, observe: None, respond: Some(the_trouble_it_has::respond), after: None },
        ModuleDef { name: "the_wait_about_a_body", priority: -999.0, observe: None, respond: Some(the_wait_about_a_body::respond), after: None },
        ModuleDef { name: "where_it_answers", priority: -999.0, observe: None, respond: Some(where_it_answers::respond), after: None },
        ModuleDef { name: "first_line_open", priority: -999.5, observe: None, respond: Some(first_line_open::respond), after: None },
        ModuleDef { name: "the_shape_of_the_answer", priority: -999.5, observe: None, respond: Some(the_shape_of_the_answer::respond), after: None },
        ModuleDef { name: "greeting", priority: -1000.0, observe: None, respond: Some(greeting::respond), after: None },
    ];
    REGISTRY
}

/// The arbiter's `choose`.
pub fn choose(text: &str, memory: &crate::state::Memory, candidates: &[crate::runtime::Candidate]) -> usize {
    arbiter::choose(text, memory, candidates)
}
