//! STUB for `bot/memory/facts.py` (owned by another workstream): only the two functions the
//! understanding engine calls (`kinds._says_what_they_are`, `kinds._a_name`), so this branch
//! compiles. The integrator replaces this file with the real port.

/// `about_themselves(said)`: the facts a line says about the person, or None.
pub fn about_themselves(said: &str) -> Option<Vec<String>> {
    let _ = said;
    None
}

/// `reads_as_a_name(word)`: the name in its spelled form, or None.
pub fn reads_as_a_name(word: &str) -> Option<String> {
    let _ = word;
    None
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

    /// Python: bot/memory/facts.py:168 `_health_words()` (never called on the trace corpus)
    pub fn _health_words() -> Value {
        crate::unported!("bot.memory.facts:_health_words")
    }

    /// Python: bot/memory/facts.py:266 `without_the_hedges(text)` (never called on the trace corpus)
    pub fn without_the_hedges(text: &str) -> String {
        crate::unported!("bot.memory.facts:without_the_hedges")
    }

    /// Python: bot/memory/facts.py:356 `_fact_value(word)`
    pub fn _fact_value(word: &str) -> String {
        crate::unported!("bot.memory.facts:_fact_value")
    }

    /// Python: bot/memory/facts.py:502 `grown_up_in_a_place(phrase)` (never called on the trace corpus)
    pub fn grown_up_in_a_place(phrase: &str) -> bool {
        crate::unported!("bot.memory.facts:grown_up_in_a_place")
    }

    /// Python: bot/memory/facts.py:511 `_grown_up_in_a_place(phrase, text)` (never called on the trace corpus)
    pub fn _grown_up_in_a_place(phrase: &str, text: &str) -> bool {
        crate::unported!("bot.memory.facts:_grown_up_in_a_place")
    }

    /// Python: bot/memory/facts.py:552 `_an_everyday_word(word)`
    pub fn _an_everyday_word(word: &str) -> bool {
        crate::unported!("bot.memory.facts:_an_everyday_word")
    }

    /// Python: bot/memory/facts.py:849 `_the_drink_they_do_not_take(text)`
    pub fn _the_drink_they_do_not_take(text: &str) -> String {
        crate::unported!("bot.memory.facts:_the_drink_they_do_not_take")
    }

    /// Python: bot/memory/facts.py:1016 `_coming_home(text, start)`
    pub fn _coming_home(text: &str, start: i64) -> bool {
        crate::unported!("bot.memory.facts:_coming_home")
    }

    /// Python: bot/memory/facts.py:1027 `_only_wanted(text, start, end)`
    pub fn _only_wanted(text: &str, start: i64, end: i64) -> bool {
        crate::unported!("bot.memory.facts:_only_wanted")
    }

    /// Python: bot/memory/facts.py:1045 `_not_theirs(animal, text, start, end)`
    pub fn _not_theirs(animal: &str, text: &str, start: i64, end: i64) -> Value {
        crate::unported!("bot.memory.facts:_not_theirs")
    }

    /// Python: bot/memory/facts.py:1053 `_a_wish_for(text, start)`
    pub fn _a_wish_for(text: &str, start: i64) -> bool {
        crate::unported!("bot.memory.facts:_a_wish_for")
    }

    /// Python: bot/memory/facts.py:1122 `_the_food_named(raw)`
    pub fn _the_food_named(raw: &str) -> String {
        crate::unported!("bot.memory.facts:_the_food_named")
    }

    /// Python: bot/memory/facts.py:1167 `_a_way_of_eating_said_with_a_verb(text)`
    pub fn _a_way_of_eating_said_with_a_verb(text: &str) -> String {
        crate::unported!("bot.memory.facts:_a_way_of_eating_said_with_a_verb")
    }

    /// Python: bot/memory/facts.py:1300 `_with_title(match)`
    pub fn _with_title(r#match: &Value) -> String {
        crate::unported!("bot.memory.facts:_with_title")
    }

    /// Python: bot/memory/facts.py:1319 `_name_at(match)`
    pub fn _name_at(r#match: &Value) -> i64 {
        crate::unported!("bot.memory.facts:_name_at")
    }

    /// Python: bot/memory/facts.py:1380 `another_animal(text, animal)`
    pub fn another_animal(text: &str, animal: &str) -> String {
        crate::unported!("bot.memory.facts:another_animal")
    }

    /// Python: bot/memory/facts.py:1429 `_a_name_for_the_pet(word, strict_ly=True)` (never called on the trace corpus)
    pub fn _a_name_for_the_pet(word: &str, strict_ly: &Value) -> Value {
        crate::unported!("bot.memory.facts:_a_name_for_the_pet")
    }

    /// Python: bot/memory/facts.py:1475 `_names_something_else(low)` (never called on the trace corpus)
    pub fn _names_something_else(low: &str) -> bool {
        crate::unported!("bot.memory.facts:_names_something_else")
    }

    /// Python: bot/memory/facts.py:1724 `vague_budget(text)`
    pub fn vague_budget(text: &str) {
        crate::unported!("bot.memory.facts:vague_budget")
    }

    /// Python: bot/memory/facts.py:1809 `_the_numbers_in_words(text)`
    pub fn _the_numbers_in_words(text: &str) -> String {
        crate::unported!("bot.memory.facts:_the_numbers_in_words")
    }

    /// Python: bot/memory/facts.py:1827 `_how_tall(text)`
    pub fn _how_tall(text: &str) -> String {
        crate::unported!("bot.memory.facts:_how_tall")
    }

    /// Python: bot/memory/facts.py:1868 `_the_date_clause(date)` (never called on the trace corpus)
    pub fn _the_date_clause(date: &Value) -> Value {
        crate::unported!("bot.memory.facts:_the_date_clause")
    }

    /// Python: bot/memory/facts.py:1873 `_a_birthday_date(after)` (never called on the trace corpus)
    pub fn _a_birthday_date(after: &str) -> Value {
        crate::unported!("bot.memory.facts:_a_birthday_date")
    }

    /// Python: bot/memory/facts.py:1898 `_weight_unit(word)`
    pub fn _weight_unit(word: &str) {
        crate::unported!("bot.memory.facts:_weight_unit")
    }

    /// Python: bot/memory/facts.py:2041 `an_in_law(word)` (never called on the trace corpus)
    pub fn an_in_law(word: &str) -> bool {
        crate::unported!("bot.memory.facts:an_in_law")
    }

    /// Python: bot/memory/facts.py:2206 `a_chained_person(text)`
    pub fn a_chained_person(text: &str) -> String {
        crate::unported!("bot.memory.facts:a_chained_person")
    }

    /// Python: bot/memory/facts.py:2256 `a_stated_number(word)` (never called on the trace corpus)
    pub fn a_stated_number(word: &str) -> Value {
        crate::unported!("bot.memory.facts:a_stated_number")
    }

    /// Python: bot/memory/facts.py:2267 `a_person_word(word)`
    pub fn a_person_word(word: &str) -> String {
        crate::unported!("bot.memory.facts:a_person_word")
    }

    /// Python: bot/memory/facts.py:2277 `_the_person(found, who, name='')` (never called on the trace corpus)
    pub fn _the_person(found: &Value, who: &str, name: &str) -> Value {
        crate::unported!("bot.memory.facts:_the_person")
    }

    /// Python: bot/memory/facts.py:2295 `_relation_facts(text)`
    pub fn _relation_facts(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_relation_facts")
    }

    /// Python: bot/memory/facts.py:2541 `_kin_pronoun(word)` (never called on the trace corpus)
    pub fn _kin_pronoun(word: &str) -> Value {
        crate::unported!("bot.memory.facts:_kin_pronoun")
    }

    /// Python: bot/memory/facts.py:2562 `_sided(word)` (never called on the trace corpus)
    pub fn _sided(word: &str) -> String {
        crate::unported!("bot.memory.facts:_sided")
    }

    /// Python: bot/memory/facts.py:2625 `_own_body_is_the_trouble(clean)`
    pub fn _own_body_is_the_trouble(clean: &str) -> bool {
        crate::unported!("bot.memory.facts:_own_body_is_the_trouble")
    }

    /// Python: bot/memory/facts.py:2642 `a_limit_on_the_plan(text)`
    pub fn a_limit_on_the_plan(text: &str) {
        crate::unported!("bot.memory.facts:a_limit_on_the_plan")
    }

    /// Python: bot/memory/facts.py:2780 `a_deed(clause)`
    pub fn a_deed(clause: &str) -> bool {
        crate::unported!("bot.memory.facts:a_deed")
    }

    /// Python: bot/memory/facts.py:2940 `a_clock_time(phrase)` (never called on the trace corpus)
    pub fn a_clock_time(phrase: &str) -> Value {
        crate::unported!("bot.memory.facts:a_clock_time")
    }

    /// Python: bot/memory/facts.py:2952 `the_clock_in(text)` (never called on the trace corpus)
    pub fn the_clock_in(text: &str) -> Value {
        crate::unported!("bot.memory.facts:the_clock_in")
    }

    /// Python: bot/memory/facts.py:2957 `a_moved_time(phrase)` (never called on the trace corpus)
    pub fn a_moved_time(phrase: &str) -> bool {
        crate::unported!("bot.memory.facts:a_moved_time")
    }

    /// Python: bot/memory/facts.py:2977 `_a_day_taken_back(text)`
    pub fn _a_day_taken_back(text: &str) {
        crate::unported!("bot.memory.facts:_a_day_taken_back")
    }

    /// Python: bot/memory/facts.py:2986 `_a_moved_time(text)`
    pub fn _a_moved_time(text: &str) {
        crate::unported!("bot.memory.facts:_a_moved_time")
    }

    /// Python: bot/memory/facts.py:3206 `a_name_is_taken_back(text)` (never called on the trace corpus)
    pub fn a_name_is_taken_back(text: &str) -> bool {
        crate::unported!("bot.memory.facts:a_name_is_taken_back")
    }

    /// Python: bot/memory/facts.py:3247 `_the_same_name(left, right)` (never called on the trace corpus)
    pub fn _the_same_name(left: &Value, right: &Value) -> bool {
        crate::unported!("bot.memory.facts:_the_same_name")
    }

    /// Python: bot/memory/facts.py:3256 `the_name_they_take_back(text, stored='')`
    pub fn the_name_they_take_back(text: &str, stored: &str) {
        crate::unported!("bot.memory.facts:the_name_they_take_back")
    }

    /// Python: bot/memory/facts.py:3340 `_the_marker_in_front(text, question=False)`
    pub fn _the_marker_in_front(text: &str, question: bool) {
        crate::unported!("bot.memory.facts:_the_marker_in_front")
    }

    /// Python: bot/memory/facts.py:3587 `_a_tail_hangs_off(pattern, then='')` (never called on the trace corpus)
    pub fn _a_tail_hangs_off(pattern: &Value, then: &Value) -> Value {
        crate::unported!("bot.memory.facts:_a_tail_hangs_off")
    }

    /// Python: bot/memory/facts.py:3615 `a_value(phrase)` (never called on the trace corpus)
    pub fn a_value(phrase: &str) -> bool {
        crate::unported!("bot.memory.facts:a_value")
    }

    /// Python: bot/memory/facts.py:3635 `reread(value, key)` (never called on the trace corpus)
    pub fn reread(value: &Value, key: &str) -> Value {
        crate::unported!("bot.memory.facts:reread")
    }

    /// Python: bot/memory/facts.py:3735 `a_place_that_employs(phrase)` (never called on the trace corpus)
    pub fn a_place_that_employs(phrase: &str) -> bool {
        crate::unported!("bot.memory.facts:a_place_that_employs")
    }

    /// Python: bot/memory/facts.py:3866 `is_a_judgement(text)` (never called on the trace corpus)
    pub fn is_a_judgement(text: &str) -> bool {
        crate::unported!("bot.memory.facts:is_a_judgement")
    }

    /// Python: bot/memory/facts.py:3902 `take_back_cut(phrase)` (never called on the trace corpus)
    pub fn take_back_cut(phrase: &str) -> Value {
        crate::unported!("bot.memory.facts:take_back_cut")
    }

    /// Python: bot/memory/facts.py:3918 `qualified_by(phrase)` (never called on the trace corpus)
    pub fn qualified_by(phrase: &str) -> Value {
        crate::unported!("bot.memory.facts:qualified_by")
    }

    /// Python: bot/memory/facts.py:3933 `_the_role(words)`
    pub fn _the_role(words: &[String]) -> String {
        crate::unported!("bot.memory.facts:_the_role")
    }

    /// Python: bot/memory/facts.py:3983 `is_a_place_of_work(frame)` (never called on the trace corpus)
    pub fn is_a_place_of_work(frame: &Value) -> bool {
        crate::unported!("bot.memory.facts:is_a_place_of_work")
    }

    /// Python: bot/memory/facts.py:3995 `is_a_work_detail(value, frame=None)` (never called on the trace corpus)
    pub fn is_a_work_detail(value: &Value, frame: &Value) -> bool {
        crate::unported!("bot.memory.facts:is_a_work_detail")
    }

    /// Python: bot/memory/facts.py:4013 `work_weight(value, frame=None)` (never called on the trace corpus)
    pub fn work_weight(value: &Value, frame: &Value) -> i64 {
        crate::unported!("bot.memory.facts:work_weight")
    }

    /// Python: bot/memory/facts.py:4025 `_finish(word)`
    pub fn _finish(word: &str) -> String {
        crate::unported!("bot.memory.facts:_finish")
    }

    /// Python: bot/memory/facts.py:4039 `without_the_moment(words)`
    pub fn without_the_moment(words: &[String]) -> Vec<String> {
        crate::unported!("bot.memory.facts:without_the_moment")
    }

    /// Python: bot/memory/facts.py:4092 `_can_end(words)` (never called on the trace corpus)
    pub fn _can_end(words: &[String]) -> bool {
        crate::unported!("bot.memory.facts:_can_end")
    }

    /// Python: bot/memory/facts.py:4105 `no_tone(text)`
    pub fn no_tone(text: &str) -> String {
        crate::unported!("bot.memory.facts:no_tone")
    }

    /// Python: bot/memory/facts.py:4127 `_date_change(text)`
    pub fn _date_change(text: &str) {
        crate::unported!("bot.memory.facts:_date_change")
    }

    /// Python: bot/memory/facts.py:4175 `a_moved_place(text)`
    pub fn a_moved_place(text: &str) -> String {
        crate::unported!("bot.memory.facts:a_moved_place")
    }

    /// Python: bot/memory/facts.py:4199 `a_date_put_right(text)`
    pub fn a_date_put_right(text: &str) -> bool {
        crate::unported!("bot.memory.facts:a_date_put_right")
    }

    /// Python: bot/memory/facts.py:4213 `the_date_put_right(text)` (never called on the trace corpus)
    pub fn the_date_put_right(text: &str) -> String {
        crate::unported!("bot.memory.facts:the_date_put_right")
    }

    /// Python: bot/memory/facts.py:4227 `a_trip_date_put_right(text)`
    pub fn a_trip_date_put_right(text: &str) -> String {
        crate::unported!("bot.memory.facts:a_trip_date_put_right")
    }

    /// Python: bot/memory/facts.py:4247 `_a_date(word)`
    pub fn _a_date(word: &str) -> bool {
        crate::unported!("bot.memory.facts:_a_date")
    }

    /// Python: bot/memory/facts.py:4277 `_tidy(word)` (never called on the trace corpus)
    pub fn _tidy(word: &str) -> Value {
        crate::unported!("bot.memory.facts:_tidy")
    }

    /// Python: bot/memory/facts.py:4282 `_tidy_a_titled_name(word)` (never called on the trace corpus)
    pub fn _tidy_a_titled_name(word: &str) -> Value {
        crate::unported!("bot.memory.facts:_tidy_a_titled_name")
    }

    /// Python: bot/memory/facts.py:4306 `_the_value_alone(phrase)` (never called on the trace corpus)
    pub fn _the_value_alone(phrase: &str) -> String {
        crate::unported!("bot.memory.facts:_the_value_alone")
    }

    /// Python: bot/memory/facts.py:4333 `_tidy_apart(word)` (never called on the trace corpus)
    pub fn _tidy_apart(word: &str) -> Value {
        crate::unported!("bot.memory.facts:_tidy_apart")
    }

    /// Python: bot/memory/facts.py:4352 `_lowercase_name_fits(text, match)`
    pub fn _lowercase_name_fits(text: &str, r#match: &Value) -> bool {
        crate::unported!("bot.memory.facts:_lowercase_name_fits")
    }

    /// Python: bot/memory/facts.py:4373 `_filler_or_intro(word)`
    pub fn _filler_or_intro(word: &str) -> bool {
        crate::unported!("bot.memory.facts:_filler_or_intro")
    }

    /// Python: bot/memory/facts.py:4391 `_squeezed(word)` (never called on the trace corpus)
    pub fn _squeezed(word: &str) -> String {
        crate::unported!("bot.memory.facts:_squeezed")
    }

    /// Python: bot/memory/facts.py:4410 `_tail_has_a_verb(words)`
    pub fn _tail_has_a_verb(words: &[String]) -> bool {
        crate::unported!("bot.memory.facts:_tail_has_a_verb")
    }

    /// Python: bot/memory/facts.py:4452 `_followers_fit(text, end, name_word=None)`
    pub fn _followers_fit(text: &str, end: i64, name_word: &str) -> bool {
        crate::unported!("bot.memory.facts:_followers_fit")
    }

    /// Python: bot/memory/facts.py:4495 `_without_asides(tail)`
    pub fn _without_asides(tail: &str) -> String {
        crate::unported!("bot.memory.facts:_without_asides")
    }

    /// Python: bot/memory/facts.py:4521 `_a_state_word(low)`
    pub fn _a_state_word(low: &str) -> bool {
        crate::unported!("bot.memory.facts:_a_state_word")
    }

    /// Python: bot/memory/facts.py:4536 `_a_degree_of_a_state(low)`
    pub fn _a_degree_of_a_state(low: &str) -> bool {
        crate::unported!("bot.memory.facts:_a_degree_of_a_state")
    }

    /// Python: bot/memory/facts.py:4549 `_a_letter_too_many(low)`
    pub fn _a_letter_too_many(low: &str) -> bool {
        crate::unported!("bot.memory.facts:_a_letter_too_many")
    }

    /// Python: bot/memory/facts.py:4555 `_sounds_like_a_state(low)`
    pub fn _sounds_like_a_state(low: &str) -> bool {
        crate::unported!("bot.memory.facts:_sounds_like_a_state")
    }

    /// Python: bot/memory/facts.py:4571 `_a_word_that_is_a_state(word)`
    pub fn _a_word_that_is_a_state(word: &str) -> bool {
        crate::unported!("bot.memory.facts:_a_word_that_is_a_state")
    }

    /// Python: bot/memory/facts.py:4587 `_a_word_not_a_name(low)`
    pub fn _a_word_not_a_name(low: &str) -> bool {
        crate::unported!("bot.memory.facts:_a_word_not_a_name")
    }

    /// Python: bot/memory/facts.py:4608 `_the_state_compared(word)`
    pub fn _the_state_compared(word: &str) {
        crate::unported!("bot.memory.facts:_the_state_compared")
    }

    /// Python: bot/memory/facts.py:4631 `_with_its_y_back(stem)` (never called on the trace corpus)
    pub fn _with_its_y_back(stem: &str) -> Option<String> {
        crate::unported!("bot.memory.facts:_with_its_y_back")
    }

    /// Python: bot/memory/facts.py:4643 `_a_letter_off_each_run(word)`
    pub fn _a_letter_off_each_run(word: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_a_letter_off_each_run")
    }

    /// Python: bot/memory/facts.py:4665 `_a_state_written_as_a_comparison_or_a_double(low)`
    pub fn _a_state_written_as_a_comparison_or_a_double(low: &str) -> bool {
        crate::unported!("bot.memory.facts:_a_state_written_as_a_comparison_or_a_double")
    }

    /// Python: bot/memory/facts.py:4690 `_name(text)`
    pub fn _name(text: &str) {
        crate::unported!("bot.memory.facts:_name")
    }

    /// Python: bot/memory/facts.py:4735 `_a_judgement_on_themselves(low)`
    pub fn _a_judgement_on_themselves(low: &str) -> bool {
        crate::unported!("bot.memory.facts:_a_judgement_on_themselves")
    }

    /// Python: bot/memory/facts.py:4749 `_name_from(pattern, text, guessed)`
    pub fn _name_from(pattern: &Value, text: &str, guessed: bool) {
        crate::unported!("bot.memory.facts:_name_from")
    }

    /// Python: bot/memory/facts.py:4838 `_first_tail(phrase, tails=JOB_TAIL)`
    pub fn _first_tail(phrase: &str, tails: &[String]) -> Value {
        crate::unported!("bot.memory.facts:_first_tail")
    }

    /// Python: bot/memory/facts.py:4853 `_job_as_a(text)`
    pub fn _job_as_a(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_job_as_a")
    }

    /// Python: bot/memory/facts.py:4880 `_is_a_judgement_phrase(words)`
    pub fn _is_a_judgement_phrase(words: &[String]) -> bool {
        crate::unported!("bot.memory.facts:_is_a_judgement_phrase")
    }

    /// Python: bot/memory/facts.py:4895 `_only_discourse_before(before)`
    pub fn _only_discourse_before(before: &str) -> bool {
        crate::unported!("bot.memory.facts:_only_discourse_before")
    }

    /// Python: bot/memory/facts.py:4901 `_the_correction_named_a_thing(text, match)`
    pub fn _the_correction_named_a_thing(text: &str, r#match: &Value) -> bool {
        crate::unported!("bot.memory.facts:_the_correction_named_a_thing")
    }

    /// Python: bot/memory/facts.py:4923 `_was_names_a_role(phrase, article=None)`
    pub fn _was_names_a_role(phrase: &str, article: &Value) -> bool {
        crate::unported!("bot.memory.facts:_was_names_a_role")
    }

    /// Python: bot/memory/facts.py:4954 `_job_and_city(text, name=None, question=False)`
    pub fn _job_and_city(text: &str, name: &str, question: bool) -> Value {
        crate::unported!("bot.memory.facts:_job_and_city")
    }

    /// Python: bot/memory/facts.py:5072 `place_value(phrase)` (never called on the trace corpus)
    pub fn place_value(phrase: &str) -> Value {
        crate::unported!("bot.memory.facts:place_value")
    }

    /// Python: bot/memory/facts.py:5127 `_the_car_name_in(phrase)`
    pub fn _the_car_name_in(phrase: &str) -> String {
        crate::unported!("bot.memory.facts:_the_car_name_in")
    }

    /// Python: bot/memory/facts.py:5148 `_car_named_in_passing(text)`
    pub fn _car_named_in_passing(text: &str) {
        crate::unported!("bot.memory.facts:_car_named_in_passing")
    }

    /// Python: bot/memory/facts.py:5161 `_car(text)`
    pub fn _car(text: &str) {
        crate::unported!("bot.memory.facts:_car")
    }

    /// Python: bot/memory/facts.py:5192 `_reads_as_a_plural(name_word, first)`
    pub fn _reads_as_a_plural(name_word: &str, first: &str) -> bool {
        crate::unported!("bot.memory.facts:_reads_as_a_plural")
    }

    /// Python: bot/memory/facts.py:5213 `_place_name(phrase)`
    pub fn _place_name(phrase: &str) -> String {
        crate::unported!("bot.memory.facts:_place_name")
    }

    /// Python: bot/memory/facts.py:5257 `_born_place(text)`
    pub fn _born_place(text: &str) {
        crate::unported!("bot.memory.facts:_born_place")
    }

    /// Python: bot/memory/facts.py:5295 `_the_person_moved(before)` (never called on the trace corpus)
    pub fn _the_person_moved(before: &str) -> bool {
        crate::unported!("bot.memory.facts:_the_person_moved")
    }

    /// Python: bot/memory/facts.py:5302 `_from_place(text)`
    pub fn _from_place(text: &str) {
        crate::unported!("bot.memory.facts:_from_place")
    }

    /// Python: bot/memory/facts.py:5345 `_places(text)`
    pub fn _places(text: &str) -> Value {
        crate::unported!("bot.memory.facts:_places")
    }

    /// Python: bot/memory/facts.py:5476 `stated_not_asked(text)`
    pub fn stated_not_asked(text: &str) -> String {
        crate::unported!("bot.memory.facts:stated_not_asked")
    }

    /// Python: bot/memory/facts.py:5524 `_the_person_is_in_it(clause)`
    pub fn _the_person_is_in_it(clause: &str) -> bool {
        crate::unported!("bot.memory.facts:_the_person_is_in_it")
    }

    /// Python: bot/memory/facts.py:5529 `subject(text, pos)`
    pub fn subject(text: &str, pos: i64) -> Vec<String> {
        crate::unported!("bot.memory.facts:subject")
    }

    /// Python: bot/memory/facts.py:5628 `_a_person_word(word)` (never called on the trace corpus)
    pub fn _a_person_word(word: &str) -> String {
        crate::unported!("bot.memory.facts:_a_person_word")
    }

    /// Python: bot/memory/facts.py:5633 `_the_person_without_a_relation(clause)`
    pub fn _the_person_without_a_relation(clause: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_the_person_without_a_relation")
    }

    /// Python: bot/memory/facts.py:5683 `_trim_filler(text)`
    pub fn _trim_filler(text: &str) -> String {
        crate::unported!("bot.memory.facts:_trim_filler")
    }

    /// Python: bot/memory/facts.py:5717 `_trim_time_tail(item)`
    pub fn _trim_time_tail(item: &str) -> String {
        crate::unported!("bot.memory.facts:_trim_time_tail")
    }

    /// Python: bot/memory/facts.py:5739 `_a_modifier_before_a_generic_noun(words)`
    pub fn _a_modifier_before_a_generic_noun(words: &[String]) -> bool {
        crate::unported!("bot.memory.facts:_a_modifier_before_a_generic_noun")
    }

    /// Python: bot/memory/facts.py:5748 `_only_stop_words(item)`
    pub fn _only_stop_words(item: &str) -> bool {
        crate::unported!("bot.memory.facts:_only_stop_words")
    }

    /// Python: bot/memory/facts.py:5754 `_allergy_items(chunk)`
    pub fn _allergy_items(chunk: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_allergy_items")
    }

    /// Python: bot/memory/facts.py:5796 `one_more(text)`
    pub fn one_more(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:one_more")
    }

    /// Python: bot/memory/facts.py:5899 `the_limit_they_share(text)`
    pub fn the_limit_they_share(text: &str) -> String {
        crate::unported!("bot.memory.facts:the_limit_they_share")
    }

    /// Python: bot/memory/facts.py:5928 `denied_before(text, start)`
    pub fn denied_before(text: &str, start: i64) -> bool {
        crate::unported!("bot.memory.facts:denied_before")
    }

    /// Python: bot/memory/facts.py:5933 `_clause(text, start)`
    pub fn _clause(text: &str, start: i64) -> String {
        crate::unported!("bot.memory.facts:_clause")
    }

    /// Python: bot/memory/facts.py:5942 `_the_clause_before(text, start, edges=4)`
    pub fn _the_clause_before(text: &str, start: i64, edges: i64) -> String {
        crate::unported!("bot.memory.facts:_the_clause_before")
    }

    /// Python: bot/memory/facts.py:5969 `_opens_a_subject(clause)`
    pub fn _opens_a_subject(clause: &str) -> bool {
        crate::unported!("bot.memory.facts:_opens_a_subject")
    }

    /// Python: bot/memory/facts.py:5977 `_around(text, start)` (never called on the trace corpus)
    pub fn _around(text: &str, start: i64) -> Value {
        crate::unported!("bot.memory.facts:_around")
    }

    /// Python: bot/memory/facts.py:5989 `_own_named(text, start, own)` (never called on the trace corpus)
    pub fn _own_named(text: &str, start: i64, own: &str) -> Value {
        crate::unported!("bot.memory.facts:_own_named")
    }

    /// Python: bot/memory/facts.py:6009 `_own_in_denial(text, own)`
    pub fn _own_in_denial(text: &str, own: &Value) -> Vec<String> {
        crate::unported!("bot.memory.facts:_own_in_denial")
    }

    /// Python: bot/memory/facts.py:6042 `given_away_one(text, own=None)`
    pub fn given_away_one(text: &str, own: &Value) -> Vec<String> {
        crate::unported!("bot.memory.facts:given_away_one")
    }

    /// Python: bot/memory/facts.py:6066 `_pointed_at(text, start, plural)`
    pub fn _pointed_at(text: &str, start: i64, plural: bool) -> Vec<String> {
        crate::unported!("bot.memory.facts:_pointed_at")
    }

    /// Python: bot/memory/facts.py:6081 `not_had(text, start)`
    pub fn not_had(text: &str, start: i64) -> bool {
        crate::unported!("bot.memory.facts:not_had")
    }

    /// Python: bot/memory/facts.py:6100 `_have_none(clause)`
    pub fn _have_none(clause: &str) -> bool {
        crate::unported!("bot.memory.facts:_have_none")
    }

    /// Python: bot/memory/facts.py:6115 `taken_back_before(text, start)`
    pub fn taken_back_before(text: &str, start: i64) -> bool {
        crate::unported!("bot.memory.facts:taken_back_before")
    }

    /// Python: bot/memory/facts.py:6139 `denials(text, own=None)`
    pub fn denials(text: &str, own: &Value) -> Value {
        crate::unported!("bot.memory.facts:denials")
    }

    /// Python: bot/memory/facts.py:6233 `their_back(text)`
    pub fn their_back(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:their_back")
    }

    /// Python: bot/memory/facts.py:6271 `_one_of_several(animals, said, own)`
    pub fn _one_of_several(animals: &[String], said: &str, own: &[String]) -> Vec<String> {
        crate::unported!("bot.memory.facts:_one_of_several")
    }

    /// Python: bot/memory/facts.py:6303 `_merge_pets(had, more)`
    pub fn _merge_pets(had: &[String], more: &[String]) -> Vec<String> {
        crate::unported!("bot.memory.facts:_merge_pets")
    }

    /// Python: bot/memory/facts.py:6323 `_the_one_they_have(said, match, own)`
    pub fn _the_one_they_have(said: &str, r#match: &Value, own: &[String]) -> Vec<String> {
        crate::unported!("bot.memory.facts:_the_one_they_have")
    }

    /// Python: bot/memory/facts.py:6334 `_only_pet(own)` (never called on the trace corpus)
    pub fn _only_pet(own: &str) -> Value {
        crate::unported!("bot.memory.facts:_only_pet")
    }

    /// Python: bot/memory/facts.py:6341 `_names_a_relative(said)` (never called on the trace corpus)
    pub fn _names_a_relative(said: &str) -> bool {
        crate::unported!("bot.memory.facts:_names_a_relative")
    }

    /// Python: bot/memory/facts.py:6345 `_may_be_an_animal(said, match, own=None, said_before=None)` (never called on the trace corpus)
    pub fn _may_be_an_animal(said: &str, r#match: &Value, own: &str, said_before: &Value) -> bool {
        crate::unported!("bot.memory.facts:_may_be_an_animal")
    }

    /// Python: bot/memory/facts.py:6368 `losses(text, own=None, said_before=None)`
    pub fn losses(text: &str, own: &Value, said_before: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:losses")
    }

    /// Python: bot/memory/facts.py:6422 `_allergies(text)`
    pub fn _allergies(text: &str) -> Value {
        crate::unported!("bot.memory.facts:_allergies")
    }

    /// Python: bot/memory/facts.py:6477 `_month_age_after(text, end)` (never called on the trace corpus)
    pub fn _month_age_after(text: &str, end: i64) -> bool {
        crate::unported!("bot.memory.facts:_month_age_after")
    }

    /// Python: bot/memory/facts.py:6542 `_the_word_then_a_verb(name, following)`
    pub fn _the_word_then_a_verb(name: &str, following: &str) -> bool {
        crate::unported!("bot.memory.facts:_the_word_then_a_verb")
    }

    /// Python: bot/memory/facts.py:6605 `_may_be_a_name(word)`
    pub fn _may_be_a_name(word: &str) -> bool {
        crate::unported!("bot.memory.facts:_may_be_a_name")
    }

    /// Python: bot/memory/facts.py:6621 `a_titled_name(word)`
    pub fn a_titled_name(word: &str) -> bool {
        crate::unported!("bot.memory.facts:a_titled_name")
    }

    /// Python: bot/memory/facts.py:6627 `_looks_like_a_name(word)`
    pub fn _looks_like_a_name(word: &str) -> bool {
        crate::unported!("bot.memory.facts:_looks_like_a_name")
    }

    /// Python: bot/memory/facts.py:6643 `_name_after_a_title(name)` (never called on the trace corpus)
    pub fn _name_after_a_title(name: &str) -> bool {
        crate::unported!("bot.memory.facts:_name_after_a_title")
    }

    /// Python: bot/memory/facts.py:6668 `_the_name_without_a_possessive(text)`
    pub fn _the_name_without_a_possessive(text: &str) -> Value {
        crate::unported!("bot.memory.facts:_the_name_without_a_possessive")
    }

    /// Python: bot/memory/facts.py:6684 `_a_relations_name(text)`
    pub fn _a_relations_name(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_a_relations_name")
    }

    /// Python: bot/memory/facts.py:6775 `figures_as_digits(text)`
    pub fn figures_as_digits(text: &str) -> String {
        crate::unported!("bot.memory.facts:figures_as_digits")
    }

    /// Python: bot/memory/facts.py:6796 `kid_ages(text)`
    pub fn kid_ages(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:kid_ages")
    }

    /// Python: bot/memory/facts.py:6849 `the_ones_are_given_one_at_a_time(text)` (never called on the trace corpus)
    pub fn the_ones_are_given_one_at_a_time(text: &str) -> bool {
        crate::unported!("bot.memory.facts:the_ones_are_given_one_at_a_time")
    }

    /// Python: bot/memory/facts.py:6856 `ages_said_one_by_one(text, count)`
    pub fn ages_said_one_by_one(text: &str, count: i64) -> Vec<String> {
        crate::unported!("bot.memory.facts:ages_said_one_by_one")
    }

    /// Python: bot/memory/facts.py:6903 `kid_count(text)`
    pub fn kid_count(text: &str) {
        crate::unported!("bot.memory.facts:kid_count")
    }

    /// Python: bot/memory/facts.py:6973 `count_of_the_people(text, memory)`
    pub fn count_of_the_people(text: &str, memory: &Memory) {
        crate::unported!("bot.memory.facts:count_of_the_people")
    }

    /// Python: bot/memory/facts.py:6998 `_the_group_in_hand_is_the_children(memory)` (never called on the trace corpus)
    pub fn _the_group_in_hand_is_the_children(memory: &Memory) -> bool {
        crate::unported!("bot.memory.facts:_the_group_in_hand_is_the_children")
    }

    /// Python: bot/memory/facts.py:7095 `_kids_named_with_ages(text)`
    pub fn _kids_named_with_ages(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_kids_named_with_ages")
    }

    /// Python: bot/memory/facts.py:7127 `_the_same_person(one, other)` (never called on the trace corpus)
    pub fn _the_same_person(one: &str, other: &str) -> bool {
        crate::unported!("bot.memory.facts:_the_same_person")
    }

    /// Python: bot/memory/facts.py:7184 `_own_states(text)`
    pub fn _own_states(text: &str) -> Value {
        crate::unported!("bot.memory.facts:_own_states")
    }

    /// Python: bot/memory/facts.py:7206 `_other_states(text)`
    pub fn _other_states(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_other_states")
    }

    /// Python: bot/memory/facts.py:7332 `_the_years_are_a_things(text)`
    pub fn _the_years_are_a_things(text: &str) -> bool {
        crate::unported!("bot.memory.facts:_the_years_are_a_things")
    }

    /// Python: bot/memory/facts.py:7356 `_children_by_year_old(text)`
    pub fn _children_by_year_old(text: &str) {
        crate::unported!("bot.memory.facts:_children_by_year_old")
    }

    /// Python: bot/memory/facts.py:7436 `_ages_held(entry)`
    pub fn _ages_held(entry: &Value) -> Value {
        crate::unported!("bot.memory.facts:_ages_held")
    }

    /// Python: bot/memory/facts.py:7444 `_keep_the_ages(found, entry)`
    pub fn _keep_the_ages(found: &Value, entry: &Value) {
        crate::unported!("bot.memory.facts:_keep_the_ages")
    }

    /// Python: bot/memory/facts.py:7466 `_other_ages(text)`
    pub fn _other_ages(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_other_ages")
    }

    /// Python: bot/memory/facts.py:7659 `_relation_before(text, pos)` (never called on the trace corpus)
    pub fn _relation_before(text: &str, pos: &Value) -> Value {
        crate::unported!("bot.memory.facts:_relation_before")
    }

    /// Python: bot/memory/facts.py:7667 `pet_appositive(text, start=0)`
    pub fn pet_appositive(text: &str, start: i64) {
        crate::unported!("bot.memory.facts:pet_appositive")
    }

    /// Python: bot/memory/facts.py:7687 `names_offered(memory)`
    pub fn names_offered(memory: &Memory) {
        crate::unported!("bot.memory.facts:names_offered")
    }

    /// Python: bot/memory/facts.py:7707 `a_name_picked(text, memory)`
    pub fn a_name_picked(text: &str, memory: &Memory) {
        crate::unported!("bot.memory.facts:a_name_picked")
    }

    /// Python: bot/memory/facts.py:7736 `bare_name(text)`
    pub fn bare_name(text: &str) {
        crate::unported!("bot.memory.facts:bare_name")
    }

    /// Python: bot/memory/facts.py:7772 `foods_ruled_out(found)`
    pub fn foods_ruled_out(found: &Value) -> Vec<String> {
        crate::unported!("bot.memory.facts:foods_ruled_out")
    }

    /// Python: bot/memory/facts.py:7825 `_other_no_drink(text)`
    pub fn _other_no_drink(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_other_no_drink")
    }

    /// Python: bot/memory/facts.py:7852 `_somebody(subject_text)` (never called on the trace corpus)
    pub fn _somebody(subject_text: &str) -> Value {
        crate::unported!("bot.memory.facts:_somebody")
    }

    /// Python: bot/memory/facts.py:7861 `_other_diets(text)`
    pub fn _other_diets(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_other_diets")
    }

    /// Python: bot/memory/facts.py:7956 `_other_conditions(text)`
    pub fn _other_conditions(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_other_conditions")
    }

    /// Python: bot/memory/facts.py:8022 `_the_job_phrase(phrase, place_job=False)`
    pub fn _the_job_phrase(phrase: &str, place_job: bool) -> String {
        crate::unported!("bot.memory.facts:_the_job_phrase")
    }

    /// Python: bot/memory/facts.py:8043 `_other_jobs(text)`
    pub fn _other_jobs(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_other_jobs")
    }

    /// Python: bot/memory/facts.py:8084 `_names_before_a_fact(text)`
    pub fn _names_before_a_fact(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_names_before_a_fact")
    }

    /// Python: bot/memory/facts.py:8103 `pet_name_only(text, lower=False)`
    pub fn pet_name_only(text: &str, lower: bool) {
        crate::unported!("bot.memory.facts:pet_name_only")
    }

    /// Python: bot/memory/facts.py:8128 `_count_before(text, start)`
    pub fn _count_before(text: &str, start: i64) -> i64 {
        crate::unported!("bot.memory.facts:_count_before")
    }

    /// Python: bot/memory/facts.py:8144 `pet_counts(text)` (never called on the trace corpus)
    pub fn pet_counts(text: &str) -> Value {
        crate::unported!("bot.memory.facts:pet_counts")
    }

    /// Python: bot/memory/facts.py:8155 `_names_after(rest, count, most=None)`
    pub fn _names_after(rest: &str, count: i64, most: Option<i64>) -> Vec<String> {
        crate::unported!("bot.memory.facts:_names_after")
    }

    /// Python: bot/memory/facts.py:8260 `_not_their_animal(text, start, end)`
    pub fn _not_their_animal(text: &str, start: i64, end: i64) -> bool {
        crate::unported!("bot.memory.facts:_not_their_animal")
    }

    /// Python: bot/memory/facts.py:8275 `_someone_elses(text)`
    pub fn _someone_elses(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_someone_elses")
    }

    /// Python: bot/memory/facts.py:8329 `_people(text)`
    pub fn _people(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_people")
    }

    /// Python: bot/memory/facts.py:8495 `_a_question(text)`
    pub fn _a_question(text: &str) -> bool {
        crate::unported!("bot.memory.facts:_a_question")
    }

    /// Python: bot/memory/facts.py:8503 `_owns(text)`
    pub fn _owns(text: &str) -> Value {
        crate::unported!("bot.memory.facts:_owns")
    }

    /// Python: bot/memory/facts.py:8534 `_the_thing_of(value)`
    pub fn _the_thing_of(value: &str) -> String {
        crate::unported!("bot.memory.facts:_the_thing_of")
    }

    /// Python: bot/memory/facts.py:8541 `_the_model(value, thing='')` (never called on the trace corpus)
    pub fn _the_model(value: &Value, thing: &str) -> String {
        crate::unported!("bot.memory.facts:_the_model")
    }

    /// Python: bot/memory/facts.py:8574 `_spell_the_model(words)` (never called on the trace corpus)
    pub fn _spell_the_model(words: &[String]) -> String {
        crate::unported!("bot.memory.facts:_spell_the_model")
    }

    /// Python: bot/memory/facts.py:8583 `_a_name_given_by_the_verb(word)` (never called on the trace corpus)
    pub fn _a_name_given_by_the_verb(word: &str) -> bool {
        crate::unported!("bot.memory.facts:_a_name_given_by_the_verb")
    }

    /// Python: bot/memory/facts.py:8603 `_the_same_animal(one, other)` (never called on the trace corpus)
    pub fn _the_same_animal(one: &str, other: &str) -> bool {
        crate::unported!("bot.memory.facts:_the_same_animal")
    }

    /// Python: bot/memory/facts.py:8608 `_with_the_breed(pet, breed)`
    pub fn _with_the_breed(pet: &Value, breed: &str) -> Value {
        crate::unported!("bot.memory.facts:_with_the_breed")
    }

    /// Python: bot/memory/facts.py:8619 `_pets(text, question=False)`
    pub fn _pets(text: &str, question: bool) -> Value {
        crate::unported!("bot.memory.facts:_pets")
    }

    /// Python: bot/memory/facts.py:8846 `_pet_age_tail(after)`
    pub fn _pet_age_tail(after: &str) {
        crate::unported!("bot.memory.facts:_pet_age_tail")
    }

    /// Python: bot/memory/facts.py:8884 `_pet_age(rest)`
    pub fn _pet_age(rest: &str) {
        crate::unported!("bot.memory.facts:_pet_age")
    }

    /// Python: bot/memory/facts.py:8916 `is_addressing(word)`
    pub fn is_addressing(word: &str) -> bool {
        crate::unported!("bot.memory.facts:is_addressing")
    }

    /// Python: bot/memory/facts.py:8921 `_names_it_aside(match, text)` (never called on the trace corpus)
    pub fn _names_it_aside(r#match: &Value, text: &str) -> bool {
        crate::unported!("bot.memory.facts:_names_it_aside")
    }

    /// Python: bot/memory/facts.py:8967 `_describes_them(word)`
    pub fn _describes_them(word: &str) -> bool {
        crate::unported!("bot.memory.facts:_describes_them")
    }

    /// Python: bot/memory/facts.py:8991 `_bare_name(word, pair=False)`
    pub fn _bare_name(word: &str, pair: bool) -> String {
        crate::unported!("bot.memory.facts:_bare_name")
    }

    /// Python: bot/memory/facts.py:9047 `a_name_could_be(word)` (never called on the trace corpus)
    pub fn a_name_could_be(word: &str) -> bool {
        crate::unported!("bot.memory.facts:a_name_could_be")
    }

    /// Python: bot/memory/facts.py:9080 `is_a_contraction(word)`
    pub fn is_a_contraction(word: &str) -> bool {
        crate::unported!("bot.memory.facts:is_a_contraction")
    }

    /// Python: bot/memory/facts.py:9095 `_not_a_name(word, strict_ly=True)`
    pub fn _not_a_name(word: &str, strict_ly: bool) -> bool {
        crate::unported!("bot.memory.facts:_not_a_name")
    }

    /// Python: bot/memory/facts.py:9119 `_object_words(rest)`
    pub fn _object_words(rest: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_object_words")
    }

    /// Python: bot/memory/facts.py:9125 `_object_plan(rest)`
    pub fn _object_plan(rest: &str) -> bool {
        crate::unported!("bot.memory.facts:_object_plan")
    }

    /// Python: bot/memory/facts.py:9138 `_plan_verb(match, rest)`
    pub fn _plan_verb(r#match: &Value, rest: &str) -> String {
        crate::unported!("bot.memory.facts:_plan_verb")
    }

    /// Python: bot/memory/facts.py:9153 `_a_plan_has_a_thing(rest)`
    pub fn _a_plan_has_a_thing(rest: &str) -> bool {
        crate::unported!("bot.memory.facts:_a_plan_has_a_thing")
    }

    /// Python: bot/memory/facts.py:9175 `_asks_about_the_plan(text, at)`
    pub fn _asks_about_the_plan(text: &str, at: i64) -> bool {
        crate::unported!("bot.memory.facts:_asks_about_the_plan")
    }

    /// Python: bot/memory/facts.py:9191 `_plan_text(text, start)`
    pub fn _plan_text(text: &str, start: i64) -> String {
        crate::unported!("bot.memory.facts:_plan_text")
    }

    /// Python: bot/memory/facts.py:9248 `_how_they_are(clause)`
    pub fn _how_they_are(clause: &str) -> bool {
        crate::unported!("bot.memory.facts:_how_they_are")
    }

    /// Python: bot/memory/facts.py:9295 `_how_their_things_are(clause)`
    pub fn _how_their_things_are(clause: &str) -> bool {
        crate::unported!("bot.memory.facts:_how_their_things_are")
    }

    /// Python: bot/memory/facts.py:9400 `_the_day_described(thing)` (never called on the trace corpus)
    pub fn _the_day_described(thing: &str) -> bool {
        crate::unported!("bot.memory.facts:_the_day_described")
    }

    /// Python: bot/memory/facts.py:9435 `_the_thing_said_after_the_day(low, day)` (never called on the trace corpus)
    pub fn _the_thing_said_after_the_day(low: &str, day: &Value) -> Value {
        crate::unported!("bot.memory.facts:_the_thing_said_after_the_day")
    }

    /// Python: bot/memory/facts.py:9459 `_appointment(text)`
    pub fn _appointment(text: &str) -> String {
        crate::unported!("bot.memory.facts:_appointment")
    }

    /// Python: bot/memory/facts.py:9560 `_coming_up(clause)`
    pub fn _coming_up(clause: &str) -> bool {
        crate::unported!("bot.memory.facts:_coming_up")
    }

    /// Python: bot/memory/facts.py:9576 `_dated_plan(text, deed=None)` (never called on the trace corpus)
    pub fn _dated_plan(text: &str, deed: &Value) -> Value {
        crate::unported!("bot.memory.facts:_dated_plan")
    }

    /// Python: bot/memory/facts.py:9620 `_the_opener_end(said)`
    pub fn _the_opener_end(said: &str) -> i64 {
        crate::unported!("bot.memory.facts:_the_opener_end")
    }

    /// Python: bot/memory/facts.py:9634 `without_the_correction_opener(text)`
    pub fn without_the_correction_opener(text: &str) -> String {
        crate::unported!("bot.memory.facts:without_the_correction_opener")
    }

    /// Python: bot/memory/facts.py:9658 `is_a_correction_opener(text)` (never called on the trace corpus)
    pub fn is_a_correction_opener(text: &str) -> bool {
        crate::unported!("bot.memory.facts:is_a_correction_opener")
    }

    /// Python: bot/memory/facts.py:9675 `_said_after_the_subject(clause)`
    pub fn _said_after_the_subject(clause: &str) -> String {
        crate::unported!("bot.memory.facts:_said_after_the_subject")
    }

    /// Python: bot/memory/facts.py:9684 `_the_verb_for(body, subject)` (never called on the trace corpus)
    pub fn _the_verb_for(body: &str, subject: &str) -> Value {
        crate::unported!("bot.memory.facts:_the_verb_for")
    }

    /// Python: bot/memory/facts.py:9700 `_with_the_verb_lent(clause, earlier)`
    pub fn _with_the_verb_lent(clause: &str, earlier: &str) -> String {
        crate::unported!("bot.memory.facts:_with_the_verb_lent")
    }

    /// Python: bot/memory/facts.py:9718 `_dated_plans(text, deed=None)`
    pub fn _dated_plans(text: &str, deed: &[String]) -> Option<String> {
        crate::unported!("bot.memory.facts:_dated_plans")
    }

    /// Python: bot/memory/facts.py:9815 `_with_the_person_joined_to_the_subject(ahead, clause)`
    pub fn _with_the_person_joined_to_the_subject(ahead: &str, clause: &str) -> String {
        crate::unported!("bot.memory.facts:_with_the_person_joined_to_the_subject")
    }

    /// Python: bot/memory/facts.py:9859 `_continued_parts(text)`
    pub fn _continued_parts(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_continued_parts")
    }

    /// Python: bot/memory/facts.py:9922 `_the_relation_named(clause)`
    pub fn _the_relation_named(clause: &str) {
        crate::unported!("bot.memory.facts:_the_relation_named")
    }

    /// Python: bot/memory/facts.py:9930 `_a_clause_about_the_same_person(text)`
    pub fn _a_clause_about_the_same_person(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_a_clause_about_the_same_person")
    }

    /// Python: bot/memory/facts.py:9974 `_self_intro_parts(text)`
    pub fn _self_intro_parts(text: &str) -> Vec<String> {
        crate::unported!("bot.memory.facts:_self_intro_parts")
    }

    /// Python: bot/memory/facts.py:9997 `_same_thing(one, other)` (never called on the trace corpus)
    pub fn _same_thing(one: &str, other: &str) -> bool {
        crate::unported!("bot.memory.facts:_same_thing")
    }

    /// Python: bot/memory/facts.py:10006 `_is_only_the_take_back(value, gone)` (never called on the trace corpus)
    pub fn _is_only_the_take_back(value: &Value, gone: &Value) -> bool {
        crate::unported!("bot.memory.facts:_is_only_the_take_back")
    }

    /// Python: bot/memory/facts.py:10069 `bot_name(text)`
    pub fn bot_name(text: &str) {
        crate::unported!("bot.memory.facts:bot_name")
    }

    /// Python: bot/memory/facts.py:10114 `a_correction_without_a_subject(text, before)`
    pub fn a_correction_without_a_subject(text: &str, before: &str) -> Value {
        crate::unported!("bot.memory.facts:a_correction_without_a_subject")
    }

    /// Python: bot/memory/facts.py:10149 `_reported_clause(text)`
    pub fn _reported_clause(text: &str) {
        crate::unported!("bot.memory.facts:_reported_clause")
    }

    /// Python: bot/memory/facts.py:10164 `_paid_by_a_period(text, end)` (never called on the trace corpus)
    pub fn _paid_by_a_period(text: &str, end: i64) -> bool {
        crate::unported!("bot.memory.facts:_paid_by_a_period")
    }

    /// Python: bot/memory/facts.py:10175 `_a_years_pay(figure)` (never called on the trace corpus)
    pub fn _a_years_pay(figure: &Value) -> bool {
        crate::unported!("bot.memory.facts:_a_years_pay")
    }

    /// Python: bot/memory/facts.py:10188 `extract(text, question=False)`
    pub fn extract(text: &str, question: bool) -> Value {
        crate::unported!("bot.memory.facts:extract")
    }

    /// Python: bot/memory/facts.py:10281 `_one_entry_per_person(entries)`
    pub fn _one_entry_per_person(entries: &Value) -> Value {
        crate::unported!("bot.memory.facts:_one_entry_per_person")
    }

    /// Python: bot/memory/facts.py:10360 `_union(one, other)` (never called on the trace corpus)
    pub fn _union(one: &str, other: &str) -> Value {
        crate::unported!("bot.memory.facts:_union")
    }

    /// Python: bot/memory/facts.py:10368 `_the_pronoun(text, pos)` (never called on the trace corpus)
    pub fn _the_pronoun(text: &str, pos: &Value) -> Value {
        crate::unported!("bot.memory.facts:_the_pronoun")
    }

    /// Python: bot/memory/facts.py:10388 `_the_person_the_pronoun_is(text, pos, pronoun)` (never called on the trace corpus)
    pub fn _the_person_the_pronoun_is(text: &str, pos: &Value, pronoun: &Value) -> Value {
        crate::unported!("bot.memory.facts:_the_person_the_pronoun_is")
    }

    /// Python: bot/memory/facts.py:10428 `_a_name_opening_the_clause(clause)` (never called on the trace corpus)
    pub fn _a_name_opening_the_clause(clause: &str) -> Value {
        crate::unported!("bot.memory.facts:_a_name_opening_the_clause")
    }

    /// Python: bot/memory/facts.py:10443 `_the_name_in_front(text, pos)` (never called on the trace corpus)
    pub fn _the_name_in_front(text: &str, pos: &Value) -> Value {
        crate::unported!("bot.memory.facts:_the_name_in_front")
    }

    /// Python: bot/memory/facts.py:10475 `_a_person(text, pos, who)`
    pub fn _a_person(text: &str, pos: i64, who: &str) -> Value {
        crate::unported!("bot.memory.facts:_a_person")
    }

    /// Python: bot/memory/facts.py:10501 `_same_person(seen, entry)` (never called on the trace corpus)
    pub fn _same_person(seen: &Value, entry: &'static Entry) -> bool {
        crate::unported!("bot.memory.facts:_same_person")
    }

    /// Python: bot/memory/facts.py:10543 `_a_name_and_a_figure(text)`
    pub fn _a_name_and_a_figure(text: &str) {
        crate::unported!("bot.memory.facts:_a_name_and_a_figure")
    }

    /// Python: bot/memory/facts.py:10631 `_tied_to_them(part, low)`
    pub fn _tied_to_them(part: &Value, low: &str) -> bool {
        crate::unported!("bot.memory.facts:_tied_to_them")
    }

    /// Python: bot/memory/facts.py:10656 `_theirs(part, low)`
    pub fn _theirs(part: &Value, low: &str) -> bool {
        crate::unported!("bot.memory.facts:_theirs")
    }

    /// Python: bot/memory/facts.py:10679 `the_part_it_hurts(text)`
    pub fn the_part_it_hurts(text: &str) {
        crate::unported!("bot.memory.facts:the_part_it_hurts")
    }

    /// Python: bot/memory/facts.py:10720 `_extract(text, question=False)`
    pub fn _extract(text: &str, question: bool) -> Value {
        crate::unported!("bot.memory.facts:_extract")
    }

}

