//! Port of PyBot's `bot/memory` package: the parts the two observers call. Each Python file
//! `bot/memory/<name>.py` is `<name>.rs` here.

pub mod a_year_of_birth;
pub mod aboutthem;
pub mod ack;
pub mod again;
pub mod already;
pub mod amounts;
pub mod another;
pub mod asked_about;
pub mod asleep;
pub mod back_again;
pub mod baking;
pub mod callback;
pub mod carried_on;
pub mod carry;
pub mod cause;
pub mod change;
pub mod choice;
pub mod company;
pub mod condition;
pub mod corrected;
pub mod counts;
pub mod deadline;
pub mod decided;
pub mod detail;
pub mod doing_it;
pub mod doubt;
pub mod earlier;
pub mod enough;
pub mod favourite;
pub mod filler;
pub mod goals;
pub mod groupages;
pub mod guest;
pub mod healing;
pub mod history;
pub mod holds;
pub mod hope;
pub mod hours;
pub mod household;
pub mod kidcount;
pub mod kinds;
pub mod limit;
pub mod limits;
pub mod lineback;
pub mod matters;
pub mod meant;
pub mod measure;
pub(crate) mod mem8_py;
pub mod more;
pub mod named;
pub mod narrow;
pub mod newborn;
pub mod not_a_trip;
pub mod not_this;
pub mod obstacle;
pub mod onset;
pub mod opening;
pub mod outfit;
pub mod outgoings;
pub mod paragraph;
pub mod passing_the_time;
pub mod pending;
pub mod people;
pub mod pile;
pub mod places;
pub mod plan;
pub mod plans;
pub mod portions;
pub mod prefer;
pub mod price;
pub mod purse;
pub mod quotes;
pub mod rates;
pub mod reject;
pub mod relations;
pub mod remedy;
pub mod reminded;
pub mod rename;
pub mod repair;
pub mod repeat;
pub mod resize;
pub mod returned;
pub mod returns;
pub mod saving;
pub mod setback;
pub mod size;
pub mod spent;
pub mod spoken;
pub mod stands;
pub mod store_lines;
pub mod store_replies;
pub mod stuck;
pub mod surname;
pub mod swap;
pub mod taking;
pub mod taste;
pub mod the_verdict;
pub mod the_years;
pub mod their_feeling;
pub mod thread;
pub mod times;
pub mod told;
pub mod too_late;
pub mod tried;
pub mod trips;
pub mod warmth;
pub mod whathurts;
pub mod when;
pub mod whereabout;
pub mod whose_body;
pub mod will_it_work;
pub mod workplace;
pub mod years_in;
pub mod dream;
pub mod facts;
pub mod recall;
pub mod topics;


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

    /// Python: bot/memory/__init__.py:89 `user(memory)`
    pub fn user(memory: &mut Memory) -> Value {
        crate::unported!("bot.memory:user")
    }

    /// Python: bot/memory/__init__.py:97 `log(memory)`
    pub fn log(memory: &mut Memory) -> Value {
        crate::unported!("bot.memory:log")
    }

    /// Python: bot/memory/__init__.py:105 `others(memory)`
    pub fn others(memory: &mut Memory) -> Vec<String> {
        crate::unported!("bot.memory:others")
    }

    /// Python: bot/memory/__init__.py:129 `_said_another(text, who)` (never called on the trace corpus)
    pub fn _said_another(text: &str, who: &str) -> bool {
        crate::unported!("bot.memory:_said_another")
    }

    /// Python: bot/memory/__init__.py:147 `_another_of_them(entry, known, text)` (never called on the trace corpus)
    pub fn _another_of_them(entry: &'static Entry, known: &Value, text: &str) -> bool {
        crate::unported!("bot.memory:_another_of_them")
    }

    /// Python: bot/memory/__init__.py:164 `remember_other(memory, entry, text='')`
    pub fn remember_other(memory: &mut Memory, entry: &Value, text: &str) {
        crate::unported!("bot.memory:remember_other")
    }

    /// Python: bot/memory/__init__.py:203 `_fold_into(memory, known, entry, text='')` (never called on the trace corpus)
    pub fn _fold_into(memory: &Memory, known: &Value, entry: &'static Entry, text: &str) -> Value {
        crate::unported!("bot.memory:_fold_into")
    }

    /// Python: bot/memory/__init__.py:271 `reads_as_a_plan(value, text='')`
    pub fn reads_as_a_plan(value: &str, text: &str) -> bool {
        crate::unported!("bot.memory:reads_as_a_plan")
    }

    /// Python: bot/memory/__init__.py:299 `_the_thing_the_plan_is(plan)`
    pub fn _the_thing_the_plan_is(plan: &str) -> String {
        crate::unported!("bot.memory:_the_thing_the_plan_is")
    }

    /// Python: bot/memory/__init__.py:307 `remember(memory, key, value, text='')`
    pub fn remember(memory: &mut Memory, key: &str, value: &Value, text: &str) {
        crate::unported!("bot.memory:remember")
    }

    /// Python: bot/memory/__init__.py:533 `forget(memory, key, values, why=None)`
    pub fn forget(memory: &mut Memory, key: &str, values: &[String], why: &str) -> Vec<String> {
        crate::unported!("bot.memory:forget")
    }

    /// Python: bot/memory/__init__.py:600 `there_is_a_chat(memory)`
    pub fn there_is_a_chat(memory: &Memory) -> bool {
        crate::unported!("bot.memory:there_is_a_chat")
    }

    /// Python: bot/memory/__init__.py:614 `recent(memory, n=6)`
    pub fn recent(memory: &Memory, n: i64) -> Vec<String> {
        crate::unported!("bot.memory:recent")
    }

    /// Python: bot/memory/__init__.py:633 `last_user(memory, n=1)`
    pub fn last_user(memory: &Memory, n: i64) -> Value {
        crate::unported!("bot.memory:last_user")
    }

    /// Python: bot/memory/__init__.py:639 `last_bot(memory)`
    pub fn last_bot(memory: &Memory) -> String {
        crate::unported!("bot.memory:last_bot")
    }

    /// Python: bot/memory/__init__.py:645 `last_module(memory)`
    pub fn last_module(memory: &Memory) {
        crate::unported!("bot.memory:last_module")
    }

    /// Python: bot/memory/__init__.py:650 `name(memory)` (never called on the trace corpus)
    pub fn name(memory: &mut Memory) -> Value {
        crate::unported!("bot.memory:name")
    }

    /// Python: bot/memory/__init__.py:724 `_takes_a_thing(word)` (never called on the trace corpus)
    pub fn _takes_a_thing(word: &str) -> bool {
        crate::unported!("bot.memory:_takes_a_thing")
    }

    /// Python: bot/memory/__init__.py:732 `_a_doing(thing)` (never called on the trace corpus)
    pub fn _a_doing(thing: &str) -> bool {
        crate::unported!("bot.memory:_a_doing")
    }

    /// Python: bot/memory/__init__.py:768 `named_days(said)` (never called on the trace corpus)
    pub fn named_days(said: &str) -> String {
        crate::unported!("bot.memory:named_days")
    }

    /// Python: bot/memory/__init__.py:784 `_a_name_at_the_front(text)` (never called on the trace corpus)
    pub fn _a_name_at_the_front(text: &str) -> bool {
        crate::unported!("bot.memory:_a_name_at_the_front")
    }

    /// Python: bot/memory/__init__.py:816 `_a_plural(word)` (never called on the trace corpus)
    pub fn _a_plural(word: &str) -> bool {
        crate::unported!("bot.memory:_a_plural")
    }

    /// Python: bot/memory/__init__.py:822 `_has_its_own_verb(thing)` (never called on the trace corpus)
    pub fn _has_its_own_verb(thing: &str) -> bool {
        crate::unported!("bot.memory:_has_its_own_verb")
    }

    /// Python: bot/memory/__init__.py:837 `appointment_sentence(text)` (never called on the trace corpus)
    pub fn appointment_sentence(text: &str) -> String {
        crate::unported!("bot.memory:appointment_sentence")
    }

    /// Python: bot/memory/__init__.py:884 `_a_journey_line(said_line)` (never called on the trace corpus)
    pub fn _a_journey_line(said_line: &str) -> bool {
        crate::unported!("bot.memory:_a_journey_line")
    }

    /// Python: bot/memory/__init__.py:894 `plan_sentence(plan, said_line=None)` (never called on the trace corpus)
    pub fn plan_sentence(plan: &str, said_line: &str) -> String {
        crate::unported!("bot.memory:plan_sentence")
    }

    /// Python: bot/memory/__init__.py:899 `_the_plan_said_back(plan, said_line=None)` (never called on the trace corpus)
    pub fn _the_plan_said_back(plan: &str, said_line: &str) -> Value {
        crate::unported!("bot.memory:_the_plan_said_back")
    }

    /// Python: bot/memory/__init__.py:987 `_mid(text)` (never called on the trace corpus)
    pub fn _mid(text: &str) -> String {
        crate::unported!("bot.memory:_mid")
    }

    /// Python: bot/memory/__init__.py:1036 `_to_the_bot(text)`
    pub fn _to_the_bot(text: &str) -> String {
        crate::unported!("bot.memory:_to_the_bot")
    }

    /// Python: bot/memory/__init__.py:1059 `_from_the_bot(text)`
    pub fn _from_the_bot(text: &str) -> String {
        crate::unported!("bot.memory:_from_the_bot")
    }

    /// Python: bot/memory/__init__.py:1063 `_places()` (never called on the trace corpus)
    pub fn _places() -> Value {
        crate::unported!("bot.memory:_places")
    }

    /// Python: bot/memory/__init__.py:1071 `_not_places()` (never called on the trace corpus)
    pub fn _not_places() -> Value {
        crate::unported!("bot.memory:_not_places")
    }

    /// Python: bot/memory/__init__.py:1109 `_outside_quotes(text, repair)`
    pub fn _outside_quotes(text: &str, repair: &Value) -> String {
        crate::unported!("bot.memory:_outside_quotes")
    }

    /// Python: bot/memory/__init__.py:1122 `without_the_apostrophe(text)`
    pub fn without_the_apostrophe(text: &str) -> String {
        crate::unported!("bot.memory:without_the_apostrophe")
    }

    /// Python: bot/memory/__init__.py:1165 `_the_relation_possessive(text)`
    pub fn _the_relation_possessive(text: &str) -> String {
        crate::unported!("bot.memory:_the_relation_possessive")
    }

    /// Python: bot/memory/__init__.py:1183 `_the_possessive_a_relation_means(phrase)` (never called on the trace corpus)
    pub fn _the_possessive_a_relation_means(phrase: &str) -> Value {
        crate::unported!("bot.memory:_the_possessive_a_relation_means")
    }

    /// Python: bot/memory/__init__.py:1207 `_with_somebody(text, start)` (never called on the trace corpus)
    pub fn _with_somebody(text: &str, start: i64) -> bool {
        crate::unported!("bot.memory:_with_somebody")
    }

    /// Python: bot/memory/__init__.py:1218 `you(text)`
    pub fn you(text: &str) -> String {
        crate::unported!("bot.memory:you")
    }

    /// Python: bot/memory/__init__.py:1227 `_flip_to_the_second_person(text)`
    pub fn _flip_to_the_second_person(text: &str) -> String {
        crate::unported!("bot.memory:_flip_to_the_second_person")
    }

    /// Python: bot/memory/__init__.py:1288 `_statement(text)`
    pub fn _statement(text: &str) -> String {
        crate::unported!("bot.memory:_statement")
    }

    /// Python: bot/memory/__init__.py:1301 `plan_line(memory, plan)`
    pub fn plan_line(memory: &mut Memory, plan: &str) -> String {
        crate::unported!("bot.memory:plan_line")
    }

    /// Python: bot/memory/__init__.py:1321 `said(memory, key)` (never called on the trace corpus)
    pub fn said(memory: &mut Memory, key: &str) -> Value {
        crate::unported!("bot.memory:said")
    }

}

