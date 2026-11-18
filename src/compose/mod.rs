//! Port of PyBot's `bot/compose` package: the parts the intent observer calls. Each Python file
//! `bot/compose/<name>.py` is `<name>.rs` here.

pub mod a_count_of_packets;
pub mod a_detail_dropped;
pub mod a_document_lives_with_an_office;
pub mod a_first_time;
pub mod a_number_the_bot_knows;
pub mod a_place_nobody_can_see;
pub mod a_thing_being_weighed;
pub mod a_thing_wants_doing;
pub mod a_trouble;
pub mod a_wait_about_a_body;
pub mod aboutness;
pub mod badnews;
pub mod breakup;
pub mod check_with_you;
pub mod checking;
pub mod childwords;
pub mod circumstance;
pub mod colour_on_the_body;
pub mod doubt;
pub mod draft;
pub mod facts;
pub mod fallback;
pub mod fallback_chain;
pub mod fallback_data;
pub mod fallback_pools;
pub mod fallback_words;
pub mod find_a_place;
pub mod finding_out;
pub mod findout;
pub mod for_the_day;
pub mod gap;
pub mod goodnews;
pub mod griefplea;
pub mod habit;
pub mod handoff;
pub mod hands;
pub mod harm;
pub mod hedge;
pub mod how_it_went;
pub mod invitation;
pub mod is_it_worth_it;
pub mod judgement;
pub mod kept_nearby;
pub mod lead;
pub mod lead_in;
pub mod leaving;
pub mod limit;
pub mod liveask;
pub mod loss;
pub mod missing_half;
pub mod morning_after;
pub mod moving;
pub mod nearby_one;
pub mod news_ask;
pub mod no_advice;
pub mod no_eyes;
pub mod no_page;
pub mod not_knowing;
pub mod nottheirbest;
pub mod occasionplan;
pub mod onething;
pub mod only_they_know;
pub mod onmybehalf;
pub mod onscreen;
pub mod onsubject;
pub mod outlook;
pub mod own_ask;
pub mod own_facts;
pub mod pace;
pub mod place_facts;
pub mod plan_ask;
pub mod plans;
pub mod pleased;
pub mod postcode;
pub mod prices;
pub mod quantity;
pub mod reaction;
pub mod reading;
pub mod recall;
pub mod receiving_end;
pub mod remark;
pub mod repair;
pub mod rotten_luck;
pub mod routes;
pub mod rule;
pub mod ruledout;
pub mod safety_doubt;
pub mod safetyask;
pub mod said_about_me;
pub mod saidby;
pub mod setting_off;
pub mod stepwords;
pub mod still;
pub mod stretches;
pub mod talktime;
pub mod talkto;
pub mod taste;
pub mod the_course_of_it;
pub mod the_rule_for_it;
pub mod the_shape_without_the_fact;
pub mod the_trouble_named;
pub mod the_turn_after;
pub mod their_day;
pub mod their_detail;
pub mod their_life;
pub mod theirperson;
pub mod thread_detail;
pub mod toknow;
pub mod tone_again;
pub mod trouble;
pub mod unseen;
pub mod urgent;
pub mod verdict;
pub mod waited_for;
pub mod ward;
pub mod what_decides_it;
pub mod what_i_do_know;
pub mod who;
pub mod who_to_see;
pub mod whole;
pub mod why;
pub mod worn;
pub mod clause;
pub mod fit;
pub mod what_it_asks;


// Stubs appended by harness/integrate_skeleton.py: the Python functions this file does not
// define yet, with signatures from the generated contract. Port them in place (move a
// function up into the file when you port it).
pub use self::skeleton_stubs::*;
#[allow(unused_imports, unused_variables, non_snake_case, dead_code, clippy::all)]
mod skeleton_stubs {
    use super::*;
    use crate::data::{self, Value};
    use crate::knowledge::entry::Entry;
    use crate::runtime::Candidate;
    use crate::state::{Memory, Turn};
    use crate::understood::Understood;

    /// Python: bot/compose/__init__.py:43 `says_it_has_lingered(clean, illness=False)`
    pub fn says_it_has_lingered(clean: &str, illness: bool) -> bool {
        crate::unported!("bot.compose:says_it_has_lingered")
    }

    /// Python: bot/compose/__init__.py:84 `negator(word)`
    pub fn negator(word: &str) -> bool {
        crate::unported!("bot.compose:negator")
    }

    /// Python: bot/compose/__init__.py:94 `turned(words)`
    pub fn turned(words: &[String]) -> bool {
        crate::unported!("bot.compose:turned")
    }

    /// Python: bot/compose/__init__.py:112 `turns_round(words)`
    pub fn turns_round(words: &[String]) -> bool {
        crate::unported!("bot.compose:turns_round")
    }

    /// Python: bot/compose/__init__.py:128 `negated(clean, phrase, window=3)`
    pub fn negated(clean: &str, phrase: &str, window: i64) -> bool {
        crate::unported!("bot.compose:negated")
    }

    /// Python: bot/compose/__init__.py:142 `nearly(clean, phrase, window=2)`
    pub fn nearly(clean: &str, phrase: &str, window: i64) -> bool {
        crate::unported!("bot.compose:nearly")
    }

    /// Python: bot/compose/__init__.py:168 `doubted(clean, window=4)`
    pub fn doubted(clean: &str, window: i64) -> bool {
        crate::unported!("bot.compose:doubted")
    }

    /// Python: bot/compose/__init__.py:202 `said_again(text)`
    pub fn said_again(text: &str) -> String {
        crate::unported!("bot.compose:said_again")
    }

    /// Python: bot/compose/__init__.py:236 `_opens_with_a_name(body)` (never called on the trace corpus)
    pub fn _opens_with_a_name(body: &str) -> bool {
        crate::unported!("bot.compose:_opens_with_a_name")
    }

    /// Python: bot/compose/__init__.py:250 `greet_with(name, reply)` (never called on the trace corpus)
    pub fn greet_with(name: &str, reply: &str) -> Value {
        crate::unported!("bot.compose:greet_with")
    }

    /// Python: bot/compose/__init__.py:289 `_asks_about_a_thing_nobody_said(option, last, said)`
    pub fn _asks_about_a_thing_nobody_said(option: &str, last: Option<&str>, said: &str) -> bool {
        crate::unported!("bot.compose:_asks_about_a_thing_nobody_said")
    }

    /// Python: bot/compose/__init__.py:310 `pick(options, memory, seed_text='')`
    pub fn pick(options: &[String], memory: &Value, seed_text: &str) -> String {
        crate::unported!("bot.compose:pick")
    }

    /// Python: bot/compose/__init__.py:359 `_name_just_given(memory)`
    pub fn _name_just_given(memory: &Memory) -> String {
        crate::unported!("bot.compose:_name_just_given")
    }

    /// Python: bot/compose/__init__.py:392 `with_name(reply, memory)`
    pub fn with_name(reply: Option<&str>, memory: &Memory) -> Option<String> {
        crate::unported!("bot.compose:with_name")
    }

    /// Python: bot/compose/__init__.py:467 `_says_one_of(text, said)` (never called on the trace corpus)
    pub fn _says_one_of(text: &str, said: &str) -> bool {
        crate::unported!("bot.compose:_says_one_of")
    }

    /// Python: bot/compose/__init__.py:483 `step_when_low(clean, memory, comfort_said)`
    pub fn step_when_low(clean: &str, memory: &Memory, comfort_said: &[String]) {
        crate::unported!("bot.compose:step_when_low")
    }

}

