//! Port of `bot/modules/arbiter.py`: picks one reply from the candidates.
//!
//! Stub until the arbiter workstream lands: the first candidate (the highest PRIORITY), which
//! is what `bot/runtime.py` does without a working arbiter.

use crate::runtime::Candidate;
use crate::state::Memory;

pub fn choose(_text: &str, _memory: &Memory, _candidates: &[Candidate]) -> usize {
    0
}


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

    /// Python: bot/modules/arbiter.py:218 `_a_past_that_is_not_there(index, candidates, text, memory)`
    pub fn _a_past_that_is_not_there(index: i64, candidates: &Value, text: &str, memory: &Memory) -> i64 {
        crate::unported!("bot.modules.arbiter:_a_past_that_is_not_there")
    }

    /// Python: bot/modules/arbiter.py:273 `_an_answer_it_already_gave(index, candidates, text, memory)`
    pub fn _an_answer_it_already_gave(index: i64, candidates: &Value, text: &str, memory: &Memory) -> i64 {
        crate::unported!("bot.modules.arbiter:_an_answer_it_already_gave")
    }

    /// Python: bot/modules/arbiter.py:350 `_a_recall_from_an_earlier_turn(text, memory)`
    pub fn _a_recall_from_an_earlier_turn(text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_a_recall_from_an_earlier_turn")
    }

    /// Python: bot/modules/arbiter.py:378 `_the_kind_line_they_gave(text, memory)`
    pub fn _the_kind_line_they_gave(text: &str, memory: &Memory) {
        crate::unported!("bot.modules.arbiter:_the_kind_line_they_gave")
    }

    /// Python: bot/modules/arbiter.py:392 `_seed(candidates)` (never called on the trace corpus)
    pub fn _seed(candidates: &[Candidate]) -> Value {
        crate::unported!("bot.modules.arbiter:_seed")
    }

    /// Python: bot/modules/arbiter.py:400 `_priority(candidate)`
    pub fn _priority(candidate: &Value) -> Value {
        crate::unported!("bot.modules.arbiter:_priority")
    }

    /// Python: bot/modules/arbiter.py:426 `_the_food_the_line_refused(text, memory)`
    pub fn _the_food_the_line_refused(text: &str, memory: &Memory) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_the_food_the_line_refused")
    }

    /// Python: bot/modules/arbiter.py:448 `_offers_the_food(reply, refused)` (never called on the trace corpus)
    pub fn _offers_the_food(reply: &str, refused: &Value) -> bool {
        crate::unported!("bot.modules.arbiter:_offers_the_food")
    }

    /// Python: bot/modules/arbiter.py:474 `_offers_what_the_line_ruled_out(candidate, text, memory)` (never called on the trace corpus)
    pub fn _offers_what_the_line_ruled_out(candidate: &Value, text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_offers_what_the_line_ruled_out")
    }

    /// Python: bot/modules/arbiter.py:496 `_the_thing_the_line_ruled_out(text, memory)`
    pub fn _the_thing_the_line_ruled_out(text: &str, memory: &Memory) -> String {
        crate::unported!("bot.modules.arbiter:_the_thing_the_line_ruled_out")
    }

    /// Python: bot/modules/arbiter.py:507 `_every_real_answer_offers(thing, candidates, index)` (never called on the trace corpus)
    pub fn _every_real_answer_offers(thing: &str, candidates: &[Candidate], index: i64) -> bool {
        crate::unported!("bot.modules.arbiter:_every_real_answer_offers")
    }

    /// Python: bot/modules/arbiter.py:533 `_the_cut_the_line_asked_for(memory, candidates)`
    pub fn _the_cut_the_line_asked_for(memory: &Memory, candidates: &Value) {
        crate::unported!("bot.modules.arbiter:_the_cut_the_line_asked_for")
    }

    /// Python: bot/modules/arbiter.py:575 `_the_figures_they_gave(memory)`
    pub fn _the_figures_they_gave(memory: &Memory) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_the_figures_they_gave")
    }

    /// Python: bot/modules/arbiter.py:588 `_says_back_one_of(reply, figures)`
    pub fn _says_back_one_of(reply: &str, figures: &[String]) -> bool {
        crate::unported!("bot.modules.arbiter:_says_back_one_of")
    }

    /// Python: bot/modules/arbiter.py:595 `_the_candidates_with_a_figure(text, memory, candidates, pool)`
    pub fn _the_candidates_with_a_figure(text: &str, memory: &Memory, candidates: &Value, pool: &Value) -> Value {
        crate::unported!("bot.modules.arbiter:_the_candidates_with_a_figure")
    }

    /// Python: bot/modules/arbiter.py:689 `_amends_a_detail(text, memory)`
    pub fn _amends_a_detail(text: &str, memory: &Memory) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_amends_a_detail")
    }

    /// Python: bot/modules/arbiter.py:704 `_the_candidates_that_hand_the_detail_back(months, candidates, pool)` (never called on the trace corpus)
    pub fn _the_candidates_that_hand_the_detail_back(months: &Value, candidates: &[Candidate], pool: &Value) -> Value {
        crate::unported!("bot.modules.arbiter:_the_candidates_that_hand_the_detail_back")
    }

    /// Python: bot/modules/arbiter.py:720 `_asks_it_back(reply)` (never called on the trace corpus)
    pub fn _asks_it_back(reply: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_asks_it_back")
    }

    /// Python: bot/modules/arbiter.py:736 `_the_candidates_that_give_the_shape(text, memory, candidates, pool)`
    pub fn _the_candidates_that_give_the_shape(text: &str, memory: &Memory, candidates: &Value, pool: &Value) -> Value {
        crate::unported!("bot.modules.arbiter:_the_candidates_that_give_the_shape")
    }

    /// Python: bot/modules/arbiter.py:794 `_the_line_asks_for_a_doing(memory)` (never called on the trace corpus)
    pub fn _the_line_asks_for_a_doing(memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_the_line_asks_for_a_doing")
    }

    /// Python: bot/modules/arbiter.py:808 `_says_what_they_already_said(reply)` (never called on the trace corpus)
    pub fn _says_what_they_already_said(reply: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_says_what_they_already_said")
    }

    /// Python: bot/modules/arbiter.py:830 `_the_numbers_in(line)`
    pub fn _the_numbers_in(line: &str) -> Value {
        crate::unported!("bot.modules.arbiter:_the_numbers_in")
    }

    /// Python: bot/modules/arbiter.py:844 `_names_the_answers_figure(reply, memory)`
    pub fn _names_the_answers_figure(reply: &str, memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_names_the_answers_figure")
    }

    /// Python: bot/modules/arbiter.py:857 `_explains_the_method(reply)` (never called on the trace corpus)
    pub fn _explains_the_method(reply: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_explains_the_method")
    }

    /// Python: bot/modules/arbiter.py:868 `_the_figure_they_asked_back(candidates, real, memory, text)`
    pub fn _the_figure_they_asked_back(candidates: &Value, real: &Value, memory: &Memory, text: &str) {
        crate::unported!("bot.modules.arbiter:_the_figure_they_asked_back")
    }

    /// Python: bot/modules/arbiter.py:943 `_the_words_the_report_names(text)` (never called on the trace corpus)
    pub fn _the_words_the_report_names(text: &str) -> Value {
        crate::unported!("bot.modules.arbiter:_the_words_the_report_names")
    }

    /// Python: bot/modules/arbiter.py:949 `_a_report_and_no_ask(text)`
    pub fn _a_report_and_no_ask(text: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_a_report_and_no_ask")
    }

    /// Python: bot/modules/arbiter.py:955 `_is_advice_about(reply, words)` (never called on the trace corpus)
    pub fn _is_advice_about(reply: &str, words: &[String]) -> bool {
        crate::unported!("bot.modules.arbiter:_is_advice_about")
    }

    /// Python: bot/modules/arbiter.py:968 `_how_much_it_names(reply, words)` (never called on the trace corpus)
    pub fn _how_much_it_names(reply: &str, words: &[String]) -> Value {
        crate::unported!("bot.modules.arbiter:_how_much_it_names")
    }

    /// Python: bot/modules/arbiter.py:974 `_the_advice_over_a_definition(candidates, pool, text, chosen)`
    pub fn _the_advice_over_a_definition(candidates: &Value, pool: &Value, text: &str, chosen: i64) {
        crate::unported!("bot.modules.arbiter:_the_advice_over_a_definition")
    }

    /// Python: bot/modules/arbiter.py:1014 `_choose(text, memory, candidates)`
    pub fn _choose(text: &str, memory: &Memory, candidates: &Value) -> i64 {
        crate::unported!("bot.modules.arbiter:_choose")
    }

    /// Python: bot/modules/arbiter.py:1636 `_a_bare_doubt_reply(candidate, text, memory)`
    pub fn _a_bare_doubt_reply(candidate: &Value, text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_a_bare_doubt_reply")
    }

    /// Python: bot/modules/arbiter.py:1660 `_places_the_figure(reply)` (never called on the trace corpus)
    pub fn _places_the_figure(reply: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_places_the_figure")
    }

    /// Python: bot/modules/arbiter.py:1673 `_the_card_that_places_the_figure(candidates, best, text, memory)`
    pub fn _the_card_that_places_the_figure(candidates: &Value, best: i64, text: &str, memory: &Memory) -> i64 {
        crate::unported!("bot.modules.arbiter:_the_card_that_places_the_figure")
    }

    /// Python: bot/modules/arbiter.py:1696 `_says_only_the_line(reply, words)`
    pub fn _says_only_the_line(reply: &str, words: &[String]) -> bool {
        crate::unported!("bot.modules.arbiter:_says_only_the_line")
    }

    /// Python: bot/modules/arbiter.py:1725 `_about_the_line(reply, words, clean='')`
    pub fn _about_the_line(reply: &str, words: &[String], clean: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_about_the_line")
    }

    /// Python: bot/modules/arbiter.py:1771 `_names_the_subject(candidate, memory, words)`
    pub fn _names_the_subject(candidate: &Value, memory: &Memory, words: &[String]) -> bool {
        crate::unported!("bot.modules.arbiter:_names_the_subject")
    }

    /// Python: bot/modules/arbiter.py:1826 `_names_the_thing(word, reply, line_words)`
    pub fn _names_the_thing(word: &str, reply: &str, line_words: &[String]) -> bool {
        crate::unported!("bot.modules.arbiter:_names_the_thing")
    }

    /// Python: bot/modules/arbiter.py:1868 `_the_same_plain_word(one, other)`
    pub fn _the_same_plain_word(one: &str, other: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_the_same_plain_word")
    }

    /// Python: bot/modules/arbiter.py:1917 `_reads_the_doubt(candidate)`
    pub fn _reads_the_doubt(candidate: &Value) -> bool {
        crate::unported!("bot.modules.arbiter:_reads_the_doubt")
    }

    /// Python: bot/modules/arbiter.py:1957 `_a_bare_yes_no(memory)`
    pub fn _a_bare_yes_no(memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_a_bare_yes_no")
    }

    /// Python: bot/modules/arbiter.py:1967 `_carried_words(memory)`
    pub fn _carried_words(memory: &Memory) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_carried_words")
    }

    /// Python: bot/modules/arbiter.py:1980 `_fit(own, carried, memory, best_priority, reply='', claimant='', text='')`
    pub fn _fit(own: &[String], carried: &[String], memory: &Memory, best_priority: i64, reply: &str, claimant: &str, text: &str) -> Value {
        crate::unported!("bot.modules.arbiter:_fit")
    }

    /// Python: bot/modules/arbiter.py:2033 `_line_topics(memory)`
    pub fn _line_topics(memory: &Memory) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_line_topics")
    }

    /// Python: bot/modules/arbiter.py:2072 `_an_answer_behind_the_ask(memory)` (never called on the trace corpus)
    pub fn _an_answer_behind_the_ask(memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_an_answer_behind_the_ask")
    }

    /// Python: bot/modules/arbiter.py:2090 `_asks_for_the_answer_again(memory, text='')`
    pub fn _asks_for_the_answer_again(memory: &Memory, text: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_asks_for_the_answer_again")
    }

    /// Python: bot/modules/arbiter.py:2128 `_the_repeat_frame()`
    pub fn _the_repeat_frame() -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_the_repeat_frame")
    }

    /// Python: bot/modules/arbiter.py:2150 `_the_store_reads_a_repeat(text, memory)`
    pub fn _the_store_reads_a_repeat(text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_the_store_reads_a_repeat")
    }

    /// Python: bot/modules/arbiter.py:2174 `_the_store_reads_an_explain_again_ask(text)`
    pub fn _the_store_reads_an_explain_again_ask(text: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_the_store_reads_an_explain_again_ask")
    }

    /// Python: bot/modules/arbiter.py:2202 `_a_repeat_to_hold(memory, text='')`
    pub fn _a_repeat_to_hold(memory: &Memory, text: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_a_repeat_to_hold")
    }

    /// Python: bot/modules/arbiter.py:2218 `_a_choice_the_answer_did_not_make(candidates, text, memory, last='')`
    pub fn _a_choice_the_answer_did_not_make(candidates: &Value, text: &str, memory: &Memory, last: &str) -> Option<i64> {
        crate::unported!("bot.modules.arbiter:_a_choice_the_answer_did_not_make")
    }

    /// Python: bot/modules/arbiter.py:2258 `_a_shorter_said_again(candidates, last)`
    pub fn _a_shorter_said_again(candidates: &Value, last: &str) {
        crate::unported!("bot.modules.arbiter:_a_shorter_said_again")
    }

    /// Python: bot/modules/arbiter.py:2277 `_said_it_already(candidates, text, memory, last='')`
    pub fn _said_it_already(candidates: &Value, text: &str, memory: &Memory, last: &str) {
        crate::unported!("bot.modules.arbiter:_said_it_already")
    }

    /// Python: bot/modules/arbiter.py:2355 `_ack_lines()`
    pub fn _ack_lines() -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_ack_lines")
    }

    /// Python: bot/modules/arbiter.py:2368 `_asks_a_question(memory)` (never called on the trace corpus)
    pub fn _asks_a_question(memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_asks_a_question")
    }

    /// Python: bot/modules/arbiter.py:2382 `_asks_for_a_judgement(text, memory=None)` (never called on the trace corpus)
    pub fn _asks_for_a_judgement(text: &str, memory: &Memory) -> Option<bool> {
        crate::unported!("bot.modules.arbiter:_asks_for_a_judgement")
    }

    /// Python: bot/modules/arbiter.py:2395 `_hands_the_decision_back(reply)`
    pub fn _hands_the_decision_back(reply: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_hands_the_decision_back")
    }

    /// Python: bot/modules/arbiter.py:2417 `_the_reply_is_the_report(candidate, text, memory)` (never called on the trace corpus)
    pub fn _the_reply_is_the_report(candidate: &Value, text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_the_reply_is_the_report")
    }

    /// Python: bot/modules/arbiter.py:2438 `_real_answers_besides(candidates, last='')`
    pub fn _real_answers_besides(candidates: &Value, last: &str) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_real_answers_besides")
    }

    /// Python: bot/modules/arbiter.py:2487 `_the_ask_after_the_facts(text)`
    pub fn _the_ask_after_the_facts(text: &str) -> String {
        crate::unported!("bot.modules.arbiter:_the_ask_after_the_facts")
    }

    /// Python: bot/modules/arbiter.py:2503 `_only_says_the_line_back(reply, text)`
    pub fn _only_says_the_line_back(reply: &str, text: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_only_says_the_line_back")
    }

    /// Python: bot/modules/arbiter.py:2522 `_the_candidates_that_answer_the_ask(text, candidates, pool)`
    pub fn _the_candidates_that_answer_the_ask(text: &str, candidates: &Value, pool: &Value) {
        crate::unported!("bot.modules.arbiter:_the_candidates_that_answer_the_ask")
    }

    /// Python: bot/modules/arbiter.py:2541 `_a_card_besides(candidates, index)` (never called on the trace corpus)
    pub fn _a_card_besides(candidates: &[Candidate], index: i64) -> bool {
        crate::unported!("bot.modules.arbiter:_a_card_besides")
    }

    /// Python: bot/modules/arbiter.py:2560 `_a_real_answer_besides(candidates, last='')`
    pub fn _a_real_answer_besides(candidates: &Value, last: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_a_real_answer_besides")
    }

    /// Python: bot/modules/arbiter.py:2573 `_only_a_hand_back_besides(candidates, last='')` (never called on the trace corpus)
    pub fn _only_a_hand_back_besides(candidates: &[Candidate], last: &str) -> Value {
        crate::unported!("bot.modules.arbiter:_only_a_hand_back_besides")
    }

    /// Python: bot/modules/arbiter.py:2611 `_names_what_they_said(reply, text, memory)` (never called on the trace corpus)
    pub fn _names_what_they_said(reply: &str, text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_names_what_they_said")
    }

    /// Python: bot/modules/arbiter.py:2646 `_one_word(one, other)` (never called on the trace corpus)
    pub fn _one_word(one: &str, other: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_one_word")
    }

    /// Python: bot/modules/arbiter.py:2690 `_the_answer_handed_back(candidates, text, memory, last)`
    pub fn _the_answer_handed_back(candidates: &Value, text: &str, memory: &Memory, last: &str) {
        crate::unported!("bot.modules.arbiter:_the_answer_handed_back")
    }

    /// Python: bot/modules/arbiter.py:2726 `_names_a_figure(text)` (never called on the trace corpus)
    pub fn _names_a_figure(text: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_names_a_figure")
    }

    /// Python: bot/modules/arbiter.py:2750 `_as_it_comes(text, memory, candidates, last='')`
    pub fn _as_it_comes(text: &str, memory: &Memory, candidates: &Value, last: &str) {
        crate::unported!("bot.modules.arbiter:_as_it_comes")
    }

    /// Python: bot/modules/arbiter.py:2884 `_a_question_about_the_bot(text, memory)`
    pub fn _a_question_about_the_bot(text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_a_question_about_the_bot")
    }

    /// Python: bot/modules/arbiter.py:2900 `_a_declaring_specialist_besides(candidates, index, memory, text='')`
    pub fn _a_declaring_specialist_besides(candidates: &Value, index: i64, memory: &Memory, text: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_a_declaring_specialist_besides")
    }

    /// Python: bot/modules/arbiter.py:2936 `_a_fresh_answer_besides(candidates, index, last)` (never called on the trace corpus)
    pub fn _a_fresh_answer_besides(candidates: &[Candidate], index: i64, last: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_a_fresh_answer_besides")
    }

    /// Python: bot/modules/arbiter.py:2947 `_claims_instead(text, memory, candidates, last='', above=None)`
    pub fn _claims_instead(text: &str, memory: &Memory, candidates: &Value, last: &str, above: i64) {
        crate::unported!("bot.modules.arbiter:_claims_instead")
    }

    /// Python: bot/modules/arbiter.py:2983 `_declares_claims_instead(module_name, text, memory)` (never called on the trace corpus)
    pub fn _declares_claims_instead(module_name: &Value, text: &str, memory: &Memory) -> Option<bool> {
        crate::unported!("bot.modules.arbiter:_declares_claims_instead")
    }

    /// Python: bot/modules/arbiter.py:2993 `_declares_as_it_comes(module_name, text, memory)`
    pub fn _declares_as_it_comes(module_name: &str, text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_declares_as_it_comes")
    }

    /// Python: bot/modules/arbiter.py:3003 `_asks_about_the_chat(module_name, text, memory)`
    pub fn _asks_about_the_chat(module_name: &str, text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_asks_about_the_chat")
    }

    /// Python: bot/modules/arbiter.py:3020 `_the_move_in_hand(candidates, real, text, last)`
    pub fn _the_move_in_hand(candidates: &Value, real: &Value, text: &str, last: &str) {
        crate::unported!("bot.modules.arbiter:_the_move_in_hand")
    }

    /// Python: bot/modules/arbiter.py:3097 `_the_whole_move(candidates, real, text, last, written='')`
    pub fn _the_whole_move(candidates: &Value, real: &Value, text: &str, last: &str, written: &str) {
        crate::unported!("bot.modules.arbiter:_the_whole_move")
    }

    /// Python: bot/modules/arbiter.py:3203 `_made_of_the_answer(reply, seen)` (never called on the trace corpus)
    pub fn _made_of_the_answer(reply: &str, seen: &Value) -> bool {
        crate::unported!("bot.modules.arbiter:_made_of_the_answer")
    }

    /// Python: bot/modules/arbiter.py:3209 `_the_line_is_news(text)`
    pub fn _the_line_is_news(text: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_the_line_is_news")
    }

    /// Python: bot/modules/arbiter.py:3223 `_announces_a_repeat(reply)`
    pub fn _announces_a_repeat(reply: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_announces_a_repeat")
    }

    /// Python: bot/modules/arbiter.py:3239 `_repeat_leads()`
    pub fn _repeat_leads() -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_repeat_leads")
    }

    /// Python: bot/modules/arbiter.py:3259 `_carries_on(words, run)` (never called on the trace corpus)
    pub fn _carries_on(words: &[String], run: &Value) -> bool {
        crate::unported!("bot.modules.arbiter:_carries_on")
    }

    /// Python: bot/modules/arbiter.py:3268 `_run_in_the_answer(words, tokens, answer)` (never called on the trace corpus)
    pub fn _run_in_the_answer(words: &[String], tokens: &[String], answer: &str) -> Value {
        crate::unported!("bot.modules.arbiter:_run_in_the_answer")
    }

    /// Python: bot/modules/arbiter.py:3307 `_a_condolence(reply)`
    pub fn _a_condolence(reply: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_a_condolence")
    }

    /// Python: bot/modules/arbiter.py:3313 `_says_the_subject(reply, line)` (never called on the trace corpus)
    pub fn _says_the_subject(reply: &str, line: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_says_the_subject")
    }

    /// Python: bot/modules/arbiter.py:3327 `_the_cause_asked_for(candidates, real, text, best)`
    pub fn _the_cause_asked_for(candidates: &Value, real: &Value, text: &str, best: i64) {
        crate::unported!("bot.modules.arbiter:_the_cause_asked_for")
    }

    /// Python: bot/modules/arbiter.py:3358 `_the_date_fact(candidates, real, text)`
    pub fn _the_date_fact(candidates: &Value, real: &Value, text: &str) {
        crate::unported!("bot.modules.arbiter:_the_date_fact")
    }

    /// Python: bot/modules/arbiter.py:3394 `_the_next_part(candidates, text, last)`
    pub fn _the_next_part(candidates: &Value, text: &str, last: &str) {
        crate::unported!("bot.modules.arbiter:_the_next_part")
    }

    /// Python: bot/modules/arbiter.py:3419 `_a_lead_line(candidates, leads, last)` (never called on the trace corpus)
    pub fn _a_lead_line(candidates: &[Candidate], leads: &Value, last: &str) -> Value {
        crate::unported!("bot.modules.arbiter:_a_lead_line")
    }

    /// Python: bot/modules/arbiter.py:3444 `_only_acknowledges(memory)`
    pub fn _only_acknowledges(memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_only_acknowledges")
    }

    /// Python: bot/modules/arbiter.py:3467 `_a_presence_check(memory)`
    pub fn _a_presence_check(memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_a_presence_check")
    }

    /// Python: bot/modules/arbiter.py:3497 `_about_the_bot(candidates, clean, last='')`
    pub fn _about_the_bot(candidates: &Value, clean: &str, last: &str) {
        crate::unported!("bot.modules.arbiter:_about_the_bot")
    }

    /// Python: bot/modules/arbiter.py:3533 `_the_venue_hours_turn(candidates, clean, last='', text='')`
    pub fn _the_venue_hours_turn(candidates: &Value, clean: &str, last: &str, text: &str) {
        crate::unported!("bot.modules.arbiter:_the_venue_hours_turn")
    }

    /// Python: bot/modules/arbiter.py:3572 `_the_bots_own_taste(candidates, clean, memory, last='')`
    pub fn _the_bots_own_taste(candidates: &Value, clean: &str, memory: &Memory, last: &str) {
        crate::unported!("bot.modules.arbiter:_the_bots_own_taste")
    }

    /// Python: bot/modules/arbiter.py:3606 `_my_workings()`
    pub fn _my_workings() -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_my_workings")
    }

    /// Python: bot/modules/arbiter.py:3614 `_declares_topics(module_name)`
    pub fn _declares_topics(module_name: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_declares_topics")
    }

    /// Python: bot/modules/arbiter.py:3623 `_reads_as_a_verdict(text, memory=None)`
    pub fn _reads_as_a_verdict(text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_reads_as_a_verdict")
    }

    /// Python: bot/modules/arbiter.py:3634 `_verdict_lines()` (never called on the trace corpus)
    pub fn _verdict_lines() -> Value {
        crate::unported!("bot.modules.arbiter:_verdict_lines")
    }

    /// Python: bot/modules/arbiter.py:3650 `_the_verdict_turn(candidates, lines)` (never called on the trace corpus)
    pub fn _the_verdict_turn(candidates: &[Candidate], lines: &[String]) -> Value {
        crate::unported!("bot.modules.arbiter:_the_verdict_turn")
    }

    /// Python: bot/modules/arbiter.py:3666 `_in_a_careful_thread(clean, memory)` (never called on the trace corpus)
    pub fn _in_a_careful_thread(clean: &str, memory: &Memory) -> Option<bool> {
        crate::unported!("bot.modules.arbiter:_in_a_careful_thread")
    }

    /// Python: bot/modules/arbiter.py:3725 `_subject_words(text)`
    pub fn _subject_words(text: &str) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_subject_words")
    }

    /// Python: bot/modules/arbiter.py:3743 `_about_the_news(reply, memory)` (never called on the trace corpus)
    pub fn _about_the_news(reply: &str, memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_about_the_news")
    }

    /// Python: bot/modules/arbiter.py:3762 `_says_the_same_thing_again(reply, last)` (never called on the trace corpus)
    pub fn _says_the_same_thing_again(reply: &str, last: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_says_the_same_thing_again")
    }

    /// Python: bot/modules/arbiter.py:3782 `_the_subject_was_answered(candidate, memory)` (never called on the trace corpus)
    pub fn _the_subject_was_answered(candidate: &Value, memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_the_subject_was_answered")
    }

    /// Python: bot/modules/arbiter.py:3808 `_the_verdict_back(candidates, last='', memory=None)` (never called on the trace corpus)
    pub fn _the_verdict_back(candidates: &[Candidate], last: &str, memory: &Memory) -> Value {
        crate::unported!("bot.modules.arbiter:_the_verdict_back")
    }

    /// Python: bot/modules/arbiter.py:3844 `_verdict_reply(candidates, text, memory, last='', clean='', best=0)`
    pub fn _verdict_reply(candidates: &Value, text: &str, memory: &Memory, last: &str, clean: &str, best: i64) {
        crate::unported!("bot.modules.arbiter:_verdict_reply")
    }

    /// Python: bot/modules/arbiter.py:3874 `_instead(text, candidates, last='', own=(), clean='')`
    pub fn _instead(text: &str, candidates: &Value, last: &str, own: &[String], clean: &str) {
        crate::unported!("bot.modules.arbiter:_instead")
    }

    /// Python: bot/modules/arbiter.py:3945 `_the_ask_clause(line)`
    pub fn _the_ask_clause(line: &str) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_the_ask_clause")
    }

    /// Python: bot/modules/arbiter.py:3967 `_the_entry_that_answers_the_ask(candidates, skip, line)`
    pub fn _the_entry_that_answers_the_ask(candidates: &Value, skip: i64, line: &str) {
        crate::unported!("bot.modules.arbiter:_the_entry_that_answers_the_ask")
    }

    /// Python: bot/modules/arbiter.py:4000 `_defers(module_name, candidates, skip, last='', own=(), clean='', text='')`
    pub fn _defers(module_name: &str, candidates: &Value, skip: i64, last: &str, own: &[String], clean: &str, text: &str) {
        crate::unported!("bot.modules.arbiter:_defers")
    }

    /// Python: bot/modules/arbiter.py:4041 `_defers_to(module_name)`
    pub fn _defers_to(module_name: &str) -> Value {
        crate::unported!("bot.modules.arbiter:_defers_to")
    }

    /// Python: bot/modules/arbiter.py:4051 `_declares_instead(module_name)`
    pub fn _declares_instead(module_name: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_declares_instead")
    }

    /// Python: bot/modules/arbiter.py:4060 `_pause_reply(candidates, pool, text, memory, last='')`
    pub fn _pause_reply(candidates: &Value, pool: &Value, text: &str, memory: &Memory, last: &str) {
        crate::unported!("bot.modules.arbiter:_pause_reply")
    }

    /// Python: bot/modules/arbiter.py:4106 `_leaving_reply(candidates, own, carried, words, clean, text, memory, last='')`
    pub fn _leaving_reply(candidates: &Value, own: &[String], carried: &[String], words: &[String], clean: &str, text: &str, memory: &Memory, last: &str) {
        crate::unported!("bot.modules.arbiter:_leaving_reply")
    }

    /// Python: bot/modules/arbiter.py:4170 `_asks(reply)` (never called on the trace corpus)
    pub fn _asks(reply: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_asks")
    }

    /// Python: bot/modules/arbiter.py:4175 `_a_sign_off(reply)` (never called on the trace corpus)
    pub fn _a_sign_off(reply: &str) -> Option<bool> {
        crate::unported!("bot.modules.arbiter:_a_sign_off")
    }

    /// Python: bot/modules/arbiter.py:4183 `_declared(module_name)`
    pub fn _declared(module_name: &str) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_declared")
    }

    /// Python: bot/modules/arbiter.py:4192 `_the_store_revises_the_answer(candidates, pool, memory, text, last='')`
    pub fn _the_store_revises_the_answer(candidates: &Value, pool: &Value, memory: &Memory, text: &str, last: &str) {
        crate::unported!("bot.modules.arbiter:_the_store_revises_the_answer")
    }

    /// Python: bot/modules/arbiter.py:4248 `_a_clause_was_cut_in_half(reply)` (never called on the trace corpus)
    pub fn _a_clause_was_cut_in_half(reply: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_a_clause_was_cut_in_half")
    }

    /// Python: bot/modules/arbiter.py:4264 `_store_correcting(candidates, pool, memory, text, last='')`
    pub fn _store_correcting(candidates: &Value, pool: &Value, memory: &Memory, text: &str, last: &str) {
        crate::unported!("bot.modules.arbiter:_store_correcting")
    }

    /// Python: bot/modules/arbiter.py:4288 `_draft_index(candidates, last, clean='')`
    pub fn _draft_index(candidates: &Value, last: &str, clean: &str) {
        crate::unported!("bot.modules.arbiter:_draft_index")
    }

    /// Python: bot/modules/arbiter.py:4342 `_a_leaving_said_plain(candidates, pool, clean, memory)`
    pub fn _a_leaving_said_plain(candidates: &Value, pool: &Value, clean: &str, memory: &Memory) {
        crate::unported!("bot.modules.arbiter:_a_leaving_said_plain")
    }

    /// Python: bot/modules/arbiter.py:4381 `_sudden_loss_reply(candidates, pool, clean)`
    pub fn _sudden_loss_reply(candidates: &Value, pool: &Value, clean: &str) {
        crate::unported!("bot.modules.arbiter:_sudden_loss_reply")
    }

    /// Python: bot/modules/arbiter.py:4414 `_the_words_for_the_child(candidates, clean, memory, last='')`
    pub fn _the_words_for_the_child(candidates: &Value, clean: &str, memory: &Memory, last: &str) {
        crate::unported!("bot.modules.arbiter:_the_words_for_the_child")
    }

    /// Python: bot/modules/arbiter.py:4441 `_a_whole_line_module_spoke(candidates)` (never called on the trace corpus)
    pub fn _a_whole_line_module_spoke(candidates: &[Candidate]) -> bool {
        crate::unported!("bot.modules.arbiter:_a_whole_line_module_spoke")
    }

    /// Python: bot/modules/arbiter.py:4457 `_the_words_for_a_child_pool()` (never called on the trace corpus)
    pub fn _the_words_for_a_child_pool() -> Value {
        crate::unported!("bot.modules.arbiter:_the_words_for_a_child_pool")
    }

    /// Python: bot/modules/arbiter.py:4467 `_the_ask_about_a_child(candidates, clean, last='')`
    pub fn _the_ask_about_a_child(candidates: &Value, clean: &str, last: &str) {
        crate::unported!("bot.modules.arbiter:_the_ask_about_a_child")
    }

    /// Python: bot/modules/arbiter.py:4531 `_the_name_they_gave(candidates, text, memory)`
    pub fn _the_name_they_gave(candidates: &Value, text: &str, memory: &Memory) {
        crate::unported!("bot.modules.arbiter:_the_name_they_gave")
    }

    /// Python: bot/modules/arbiter.py:4554 `_about_the_person_the_line_names(candidates, pool, clean, last='')`
    pub fn _about_the_person_the_line_names(candidates: &Value, pool: &Value, clean: &str, last: &str) {
        crate::unported!("bot.modules.arbiter:_about_the_person_the_line_names")
    }

    /// Python: bot/modules/arbiter.py:4597 `_a_knowledge_entry_has_it(candidates)` (never called on the trace corpus)
    pub fn _a_knowledge_entry_has_it(candidates: &[Candidate]) -> bool {
        crate::unported!("bot.modules.arbiter:_a_knowledge_entry_has_it")
    }

    /// Python: bot/modules/arbiter.py:4611 `_hurting_reply(candidates, pool, clean)`
    pub fn _hurting_reply(candidates: &Value, pool: &Value, clean: &str) {
        crate::unported!("bot.modules.arbiter:_hurting_reply")
    }

    /// Python: bot/modules/arbiter.py:4636 `_story_goes_on_reply(text, memory, candidates, last='')`
    pub fn _story_goes_on_reply(text: &str, memory: &Memory, candidates: &Value, last: &str) {
        crate::unported!("bot.modules.arbiter:_story_goes_on_reply")
    }

    /// Python: bot/modules/arbiter.py:4680 `_a_story_opened(memory)` (never called on the trace corpus)
    pub fn _a_story_opened(memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_a_story_opened")
    }

    /// Python: bot/modules/arbiter.py:4721 `_a_refusal(text)` (never called on the trace corpus)
    pub fn _a_refusal(text: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_a_refusal")
    }

    /// Python: bot/modules/arbiter.py:4726 `_is_loss_reply_in(candidates, pool, pools)` (never called on the trace corpus)
    pub fn _is_loss_reply_in(candidates: &[Candidate], pool: &Value, pools: &Value) -> Value {
        crate::unported!("bot.modules.arbiter:_is_loss_reply_in")
    }

    /// Python: bot/modules/arbiter.py:4735 `_composer_reading(clean, memory)`
    pub fn _composer_reading(clean: &str, memory: &Memory) -> Value {
        crate::unported!("bot.modules.arbiter:_composer_reading")
    }

    /// Python: bot/modules/arbiter.py:4752 `_carries_the_answer(reply, memory)`
    pub fn _carries_the_answer(reply: &str, memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_carries_the_answer")
    }

    /// Python: bot/modules/arbiter.py:4768 `_a_check_on_the_answer(reply)`
    pub fn _a_check_on_the_answer(reply: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_a_check_on_the_answer")
    }

    /// Python: bot/modules/arbiter.py:4784 `_opens_on_a_job(reply)`
    pub fn _opens_on_a_job(reply: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_opens_on_a_job")
    }

    /// Python: bot/modules/arbiter.py:4795 `_is_a_generic_step(reply)`
    pub fn _is_a_generic_step(reply: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_is_a_generic_step")
    }

    /// Python: bot/modules/arbiter.py:4808 `_named_topics(memory)`
    pub fn _named_topics(memory: &Memory) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_named_topics")
    }

    /// Python: bot/modules/arbiter.py:4832 `_thread_index(candidates, real, own, target, last, care_about='', memory=None, line='')` (never called on the trace corpus)
    pub fn _thread_index(candidates: &[Candidate], real: &Value, own: &str, target: &Value, last: &str, care_about: &Value, memory: &Memory, line: &str) -> Value {
        crate::unported!("bot.modules.arbiter:_thread_index")
    }

    /// Python: bot/modules/arbiter.py:4906 `_answers_this_line(reply)`
    pub fn _answers_this_line(reply: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_answers_this_line")
    }

    /// Python: bot/modules/arbiter.py:4922 `_content(reply)`
    pub fn _content(reply: Option<&str>) -> String {
        crate::unported!("bot.modules.arbiter:_content")
    }

    /// Python: bot/modules/arbiter.py:4927 `_echoes(reply, last, announced=False)`
    pub fn _echoes(reply: &str, last: &str, announced: bool) -> bool {
        crate::unported!("bot.modules.arbiter:_echoes")
    }

    /// Python: bot/modules/arbiter.py:4976 `_the_answer_again_with_a_tail(reply, last)`
    pub fn _the_answer_again_with_a_tail(reply: &str, last: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_the_answer_again_with_a_tail")
    }

    /// Python: bot/modules/arbiter.py:5007 `_the_quoted_repeat(reply)`
    pub fn _the_quoted_repeat(reply: &str) {
        crate::unported!("bot.modules.arbiter:_the_quoted_repeat")
    }

    /// Python: bot/modules/arbiter.py:5052 `_points_back(reply)`
    pub fn _points_back(reply: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_points_back")
    }

    /// Python: bot/modules/arbiter.py:5057 `_a_run_of(previous, run)`
    pub fn _a_run_of(previous: &[String], run: &[String]) -> bool {
        crate::unported!("bot.modules.arbiter:_a_run_of")
    }

    /// Python: bot/modules/arbiter.py:5064 `_the_answer_written_out(memory)`
    pub fn _the_answer_written_out(memory: &Memory) -> String {
        crate::unported!("bot.modules.arbiter:_the_answer_written_out")
    }

    /// Python: bot/modules/arbiter.py:5072 `_last_reply(memory)`
    pub fn _last_reply(memory: &Memory) -> String {
        crate::unported!("bot.modules.arbiter:_last_reply")
    }

    /// Python: bot/modules/arbiter.py:5079 `_said_before(memory, text)`
    pub fn _said_before(memory: &Memory, text: &str) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_said_before")
    }

    /// Python: bot/modules/arbiter.py:5107 `_about(text, memory)`
    pub fn _about(text: &str, memory: &Memory) -> Value {
        crate::unported!("bot.modules.arbiter:_about")
    }

    /// Python: bot/modules/arbiter.py:5134 `_strings(value)`
    pub fn _strings(value: &[String]) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_strings")
    }

    /// Python: bot/modules/arbiter.py:5143 `_label(value)`
    pub fn _label(value: &str) -> String {
        crate::unported!("bot.modules.arbiter:_label")
    }

    /// Python: bot/modules/arbiter.py:5149 `_labels(value)`
    pub fn _labels(value: &Value) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_labels")
    }

    /// Python: bot/modules/arbiter.py:5182 `_stored(record)`
    pub fn _stored(record: &Value) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_stored")
    }

    /// Python: bot/modules/arbiter.py:5194 `_read_topic(text)`
    pub fn _read_topic(text: &str) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_read_topic")
    }

    /// Python: bot/modules/arbiter.py:5213 `_answered(module_name)`
    pub fn _answered(module_name: &str) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_answered")
    }

    /// Python: bot/modules/arbiter.py:5223 `_match(module_name, own, carried, words, clean)`
    pub fn _match(module_name: &str, own: &[String], carried: &[String], words: &[String], clean: &str) -> i64 {
        crate::unported!("bot.modules.arbiter:_match")
    }

    /// Python: bot/modules/arbiter.py:5248 `_the_subject_words(text)`
    pub fn _the_subject_words(text: &str) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_the_subject_words")
    }

    /// Python: bot/modules/arbiter.py:5257 `_the_lines_own_words(text, memory)`
    pub fn _the_lines_own_words(text: &str, memory: &Memory) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_the_lines_own_words")
    }

    /// Python: bot/modules/arbiter.py:5270 `_the_forms_of(word)`
    pub fn _the_forms_of(word: &str) -> Vec<String> {
        crate::unported!("bot.modules.arbiter:_the_forms_of")
    }

    /// Python: bot/modules/arbiter.py:5284 `_the_same_word(said, word)`
    pub fn _the_same_word(said: &str, word: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_the_same_word")
    }

    /// Python: bot/modules/arbiter.py:5289 `_names_a_word_of(a_reply, words)`
    pub fn _names_a_word_of(a_reply: &str, words: &[String]) -> bool {
        crate::unported!("bot.modules.arbiter:_names_a_word_of")
    }

    /// Python: bot/modules/arbiter.py:5295 `_a_card_names_the_line(candidates, index, text, memory)`
    pub fn _a_card_names_the_line(candidates: &Value, index: i64, text: &str, memory: &Memory) -> bool {
        crate::unported!("bot.modules.arbiter:_a_card_names_the_line")
    }

    /// Python: bot/modules/arbiter.py:5317 `_a_detail_added_to_the_news(candidates, pool, text, memory)`
    pub fn _a_detail_added_to_the_news(candidates: &Value, pool: &Value, text: &str, memory: &Memory) {
        crate::unported!("bot.modules.arbiter:_a_detail_added_to_the_news")
    }

    /// Python: bot/modules/arbiter.py:5337 `_shares_a_stem(one, other)`
    pub fn _shares_a_stem(one: &str, other: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_shares_a_stem")
    }

    /// Python: bot/modules/arbiter.py:5349 `_the_word_the_line_added(text, last)`
    pub fn _the_word_the_line_added(text: &str, last: &str) -> String {
        crate::unported!("bot.modules.arbiter:_the_word_the_line_added")
    }

    /// Python: bot/modules/arbiter.py:5366 `_says_nothing_new(reply, last)`
    pub fn _says_nothing_new(reply: &str, last: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_says_nothing_new")
    }

    /// Python: bot/modules/arbiter.py:5385 `_names_the_word(a_reply, word)`
    pub fn _names_the_word(a_reply: &str, word: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_names_the_word")
    }

    /// Python: bot/modules/arbiter.py:5390 `_the_answer_that_names_what_the_line_added(candidates, pool, text, last)`
    pub fn _the_answer_that_names_what_the_line_added(candidates: &Value, pool: &Value, text: &str, last: &str) {
        crate::unported!("bot.modules.arbiter:_the_answer_that_names_what_the_line_added")
    }

    /// Python: bot/modules/arbiter.py:5421 `_a_card_that_names_the_line(candidates, pool, text, memory)`
    pub fn _a_card_that_names_the_line(candidates: &Value, pool: &Value, text: &str, memory: &Memory) -> Option<i64> {
        crate::unported!("bot.modules.arbiter:_a_card_that_names_the_line")
    }

    /// Python: bot/modules/arbiter.py:5484 `_a_line_asking_for_plainer_words(text)`
    pub fn _a_line_asking_for_plainer_words(text: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_a_line_asking_for_plainer_words")
    }

    /// Python: bot/modules/arbiter.py:5499 `_says_the_answer_again(reply, last)` (never called on the trace corpus)
    pub fn _says_the_answer_again(reply: &str, last: &str) -> bool {
        crate::unported!("bot.modules.arbiter:_says_the_answer_again")
    }

    /// Python: bot/modules/arbiter.py:5511 `_a_plainer_reading_of_the_same_answer(candidates, best, text, memory)`
    pub fn _a_plainer_reading_of_the_same_answer(candidates: &Value, best: i64, text: &str, memory: &Memory) {
        crate::unported!("bot.modules.arbiter:_a_plainer_reading_of_the_same_answer")
    }

}

