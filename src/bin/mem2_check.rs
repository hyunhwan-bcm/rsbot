//! Check the MEM2 ports (facts, again, corrected, rename) function by function against the calls
//! PyBot made (`harness/mem2_record.py` output).
//!
//!     RSBOT_TODAY=2026-11-17 mem2_check CALLS.jsonl.gz [--fn SUBSTR] [--show K]
//!
//! Per function: calls that give PyBot's result, calls that differ, calls that reached a
//! function another workstream has not ported yet (UNPORTED), and calls whose arguments the
//! checker does not rebuild (a regex match, a generator).
use rsbot::data::Value;
use rsbot::memory::facts as f;
use rsbot::py::text as pt;
use rsbot::state::Memory;
use serde_json::{json, Value as J};
use std::collections::BTreeMap;
use std::io::BufRead;

/// Why a call was not compared.
enum Skip {
    Shape,
}

/// A recorded call's arguments, read positionally or by keyword.
struct Args<'a> {
    pos: &'a [J],
    kw: &'a serde_json::Map<String, J>,
}

impl<'a> Args<'a> {
    fn get(&self, i: usize, name: &str) -> Option<&'a J> {
        self.pos.get(i).or_else(|| self.kw.get(name))
    }
    fn s(&self, i: usize, name: &str) -> &'a str {
        self.get(i, name).and_then(J::as_str).unwrap_or("")
    }
    fn b(&self, i: usize, name: &str, default: bool) -> bool {
        match self.get(i, name) {
            Some(J::Bool(b)) => *b,
            Some(J::Null) | None => default,
            Some(J::String(s)) => !s.is_empty(),
            Some(J::Number(n)) => n.as_f64() != Some(0.0),
            Some(J::Array(a)) => !a.is_empty(),
            Some(J::Object(o)) => !o.is_empty(),
        }
    }
    fn n(&self, i: usize, name: &str) -> Result<i64, Skip> {
        self.get(i, name).and_then(J::as_i64).ok_or(Skip::Shape)
    }
    fn v(&self, i: usize, name: &str) -> Value {
        self.get(i, name).map(to_value).unwrap_or(Value::None)
    }
    fn strs(&self, i: usize, name: &str) -> Vec<String> {
        self.get(i, name)
            .and_then(J::as_array)
            .map(|a| a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect())
            .unwrap_or_default()
    }
    fn vals(&self, i: usize, name: &str) -> Vec<Value> {
        self.v(i, name).items().to_vec()
    }
    /// A Python position (code points) in `text` as a byte offset.
    fn pos_in(&self, text: &str, i: usize, name: &str) -> Result<usize, Skip> {
        let n = self.n(i, name)?;
        if n < 0 {
            return Err(Skip::Shape);
        }
        Ok(pt::byte_at(text, n as usize))
    }
    fn memory(&self, i: usize, name: &str) -> Memory {
        match self.get(i, name) {
            Some(J::Object(m)) => Memory::from_plain(m),
            _ => Memory::new(),
        }
    }
}

/// A recorded value as a `Value` (objects as dicts; the recorder's tags refused).
fn to_value(j: &J) -> Value {
    Value::from_plain(j)
}

fn has_tag(j: &J) -> bool {
    match j {
        J::Object(o) if o.contains_key("__re__") => false,
        J::Object(o) => o.contains_key("__match__") || o.contains_key("__opaque__") || o.values().any(has_tag),
        J::Array(a) => a.iter().any(has_tag),
        _ => false,
    }
}

trait ToJ {
    fn j(self) -> J;
}
impl ToJ for String {
    fn j(self) -> J {
        J::String(self)
    }
}
impl ToJ for &str {
    fn j(self) -> J {
        J::String(self.to_string())
    }
}
impl ToJ for bool {
    fn j(self) -> J {
        J::Bool(self)
    }
}
impl ToJ for i64 {
    fn j(self) -> J {
        json!(self)
    }
}
impl ToJ for Value {
    fn j(self) -> J {
        self.to_plain()
    }
}
impl<T: ToJ> ToJ for Option<T> {
    fn j(self) -> J {
        match self {
            Some(x) => x.j(),
            None => J::Null,
        }
    }
}
impl<T: ToJ> ToJ for Vec<T> {
    fn j(self) -> J {
        J::Array(self.into_iter().map(ToJ::j).collect())
    }
}
impl<A: ToJ, B: ToJ> ToJ for (A, B) {
    fn j(self) -> J {
        J::Array(vec![self.0.j(), self.1.j()])
    }
}
impl<A: ToJ, B: ToJ, C: ToJ> ToJ for (A, B, C) {
    fn j(self) -> J {
        J::Array(vec![self.0.j(), self.1.j(), self.2.j()])
    }
}

fn r<T: ToJ>(x: T) -> Result<J, Skip> {
    Ok(x.j())
}

/// Run the Rust port of one recorded call. `Ok(None)` means the port has no entry for it.
fn call(name: &str, a: &Args) -> Option<Result<J, Skip>> {
    let fname = name.rsplit(':').next().unwrap_or(name);
    let module = name.split(':').next().unwrap_or("");
    match module {
        "bot.memory.again" => return call_again(fname, a),
        "bot.memory.corrected" => return call_corrected(fname, a),
        "bot.memory.rename" => return call_rename(fname, a),
        "bot.memory.facts" => {}
        _ => return None,
    }
    let text = a.s(0, "text");
    Some(match fname {
        "_health_words" => r(f::_health_words()),
        "without_the_hedges" => r(f::without_the_hedges(text)),
        "_fact_value" => r(f::_fact_value(a.s(0, "word"))),
        "grown_up_in_a_place" => r(f::grown_up_in_a_place(a.s(0, "phrase"))),
        "_grown_up_in_a_place" => r(f::_grown_up_in_a_place(a.s(0, "phrase"), a.s(1, "text"))),
        "_an_everyday_word" => r(f::_an_everyday_word(a.s(0, "word"))),
        "_the_drink_they_do_not_take" => r(f::_the_drink_they_do_not_take(text)),
        "_coming_home" => a.pos_in(text, 1, "start").and_then(|s| r(f::_coming_home(text, s))),
        "_only_wanted" => (|| r(f::_only_wanted(text, a.pos_in(text, 1, "start")?, a.pos_in(text, 2, "end")?)))(),
        "_not_theirs" => (|| {
            let t = a.s(1, "text");
            r(f::_not_theirs(a.s(0, "animal"), t, a.pos_in(t, 2, "start")?, a.pos_in(t, 3, "end")?))
        })(),
        "_a_wish_for" => a.pos_in(text, 1, "start").and_then(|s| r(f::_a_wish_for(text, s))),
        "_the_food_named" => r(f::_the_food_named(a.s(0, "raw"))),
        "_a_way_of_eating_said_with_a_verb" => r(f::_a_way_of_eating_said_with_a_verb(text)),
        "another_animal" => r(f::another_animal(text, a.s(1, "animal"))),
        "_a_name_for_the_pet" => r(f::_a_name_for_the_pet(a.s(0, "word"), a.b(1, "strict_ly", true))),
        "_names_something_else" => r(f::_names_something_else(a.s(0, "low"))),
        "reads_as_a_name" => r(f::reads_as_a_name(a.s(0, "word"))),
        "vague_budget" => r(f::vague_budget(text)),
        "_the_numbers_in_words" => r(f::_the_numbers_in_words(text)),
        "_how_tall" => r(f::_how_tall(text)),
        "_the_date_clause" => r(f::_the_date_clause(a.s(0, "date"))),
        "_a_birthday_date" => r(f::_a_birthday_date(a.s(0, "after"))),
        "_weight_unit" => r(f::_weight_unit(a.s(0, "word"))),
        "an_in_law" => r(f::an_in_law(a.s(0, "word"))),
        "a_chained_person" => r(f::a_chained_person(text)),
        "a_stated_number" => r(f::a_stated_number(a.s(0, "word"))),
        "a_person_word" => r(f::a_person_word(a.s(0, "word"))),
        "_the_person" => {
            let found = a.vals(0, "found");
            let i = f::_the_person(&found, a.s(1, "who"), a.s(2, "name"));
            r(i.map(|i| found[i].clone()))
        }
        "_relation_facts" => r(f::_relation_facts(text)),
        "_kin_pronoun" => r(f::_kin_pronoun(a.s(0, "word"))),
        "_sided" => r(f::_sided(a.s(0, "word"))),
        "_own_body_is_the_trouble" => r(f::_own_body_is_the_trouble(a.s(0, "clean"))),
        "a_limit_on_the_plan" => r(f::a_limit_on_the_plan(text)),
        "a_deed" => r(f::a_deed(a.s(0, "clause"))),
        "a_clock_time" => r(f::a_clock_time(a.s(0, "phrase"))),
        "a_moved_time" => r(f::a_moved_time(a.s(0, "phrase"))),
        "_union" => r(f::_union(&a.vals(0, "one"), &a.vals(1, "other"))),
        "_the_pronoun" => a.pos_in(text, 1, "pos").and_then(|p| r(f::_the_pronoun(text, p))),
        "_the_person_the_pronoun_is" => {
            a.pos_in(text, 1, "pos").and_then(|p| r(f::_the_person_the_pronoun_is(text, p, a.s(2, "pronoun"))))
        }
        "_a_name_opening_the_clause" => r(f::_a_name_opening_the_clause(a.s(0, "clause"))),
        "_the_name_in_front" => a.pos_in(text, 1, "pos").and_then(|p| r(f::_the_name_in_front(text, p))),
        "_a_person" => a.pos_in(text, 1, "pos").and_then(|p| r(f::_a_person(text, p, a.s(2, "who")))),
        "_same_person" => r(f::_same_person(&a.v(0, "seen"), &a.v(1, "entry"))),
        "_tidy" => r(f::_tidy(a.s(0, "word"))),
        "_tidy_a_titled_name" => r(f::_tidy_a_titled_name(a.s(0, "word"))),
        "a_name_is_taken_back" => r(f::a_name_is_taken_back(text)),
        "_the_same_name" => r(f::_the_same_name(a.s(0, "left"), a.s(1, "right"))),
        "the_name_they_take_back" => r(f::the_name_they_take_back(text, a.s(1, "stored"))),
        "_the_marker_in_front" => r(f::_the_marker_in_front(text, a.b(1, "question", false))),
        "a_value" => r(f::a_value(a.s(0, "phrase"))),
        "reread" => r(f::reread(&a.v(0, "value"), a.s(1, "key"))),
        "a_place_that_employs" => r(f::a_place_that_employs(a.s(0, "phrase"))),
        "is_a_judgement" => r(f::is_a_judgement(text)),
        "take_back_cut" => r(f::take_back_cut(a.s(0, "phrase"))),
        "qualified_by" => r(f::qualified_by(a.s(0, "phrase"))),
        "_the_role" => r(f::_the_role(&a.strs(0, "words"))),
        "is_a_place_of_work" => r(f::is_a_place_of_work(&a.v(0, "frame"))),
        "is_a_work_detail" => r(f::is_a_work_detail(&a.v(0, "value"), &a.v(1, "frame"))),
        "work_weight" => r(f::work_weight(&a.v(0, "value"), &a.v(1, "frame"))),
        "_finish" => r(f::_finish(a.s(0, "word"))),
        "without_the_moment" => r(f::without_the_moment(&a.strs(0, "words"))),
        "_can_end" => r(f::_can_end(&a.strs(0, "words"))),
        "no_tone" => r(f::no_tone(text)),
        "_date_change" => r(f::_date_change(text)),
        "a_moved_place" => r(f::a_moved_place(text)),
        "a_date_put_right" => r(f::a_date_put_right(text)),
        "the_date_put_right" => r(f::the_date_put_right(text)),
        "a_trip_date_put_right" => r(f::a_trip_date_put_right(text)),
        "_a_date" => r(f::_a_date(a.s(0, "word"))),
        "_the_value_alone" => r(f::_the_value_alone(a.s(0, "phrase"))),
        "_tidy_apart" => r(f::_tidy_apart(a.s(0, "word"))),
        "_filler_or_intro" => r(f::_filler_or_intro(a.s(0, "word"))),
        "_squeezed" => r(f::_squeezed(a.s(0, "word"))),
        "_tail_has_a_verb" => r(f::_tail_has_a_verb(&a.strs(0, "words"))),
        "_followers_fit" => {
            a.pos_in(text, 1, "end").and_then(|e| r(f::_followers_fit(text, e, a.s(2, "name_word"))))
        }
        "_without_asides" => r(f::_without_asides(a.s(0, "tail"))),
        "_a_state_word" => r(f::_a_state_word(a.s(0, "low"))),
        "_a_degree_of_a_state" => r(f::_a_degree_of_a_state(a.s(0, "low"))),
        "_a_letter_too_many" => r(f::_a_letter_too_many(a.s(0, "low"))),
        "_sounds_like_a_state" => r(f::_sounds_like_a_state(a.s(0, "low"))),
        "_a_word_that_is_a_state" => r(f::_a_word_that_is_a_state(a.s(0, "word"))),
        "_a_word_not_a_name" => r(f::_a_word_not_a_name(a.s(0, "low"))),
        "_the_state_compared" => r(f::_the_state_compared(a.s(0, "word"))),
        "_with_its_y_back" => r(f::_with_its_y_back(a.s(0, "stem"))),
        "_a_letter_off_each_run" => r(f::_a_letter_off_each_run(a.s(0, "word"))),
        "_a_state_written_as_a_comparison_or_a_double" => r(f::_a_state_written_as_a_comparison_or_a_double(a.s(0, "low"))),
        "_name" => r(f::_name(text)),
        "_a_judgement_on_themselves" => r(f::_a_judgement_on_themselves(a.s(0, "low"))),
        "_name_from" => match a.get(0, "pattern") {
            Some(J::Object(o)) if o.contains_key("__re__") => {
                let p = &o["__re__"];
                let re = rsbot::py::re::Re::new(p[0].as_str().unwrap(), p[1].as_u64().unwrap() as u32);
                r(f::_name_from(&re, a.s(1, "text"), a.b(2, "guessed", false)))
            }
            _ => Err(Skip::Shape),
        },
        "_first_tail" => {
            let phrase = a.s(0, "phrase");
            let tails = if a.get(1, "tails").is_some() { a.strs(1, "tails") } else { rsbot::data::strs("bot.memory.facts.JOB_TAIL") };
            r(f::_first_tail(phrase, &tails).map(|(at, t)| (pt::char_index(phrase, at) as i64, t)))
        }
        "_is_a_judgement_phrase" => r(f::_is_a_judgement_phrase(&a.strs(0, "words"))),
        "_only_discourse_before" => r(f::_only_discourse_before(a.s(0, "before"))),
        "_was_names_a_role" => r(f::_was_names_a_role(a.s(0, "phrase"), a.b(1, "article", false))),
        "_job_and_city" => r(f::_job_and_city(text, a.s(1, "name"), a.b(2, "question", false))),
        "place_value" => r(f::place_value(a.s(0, "phrase"))),
        "_the_car_name_in" => r(f::_the_car_name_in(a.s(0, "phrase"))),
        "_car_named_in_passing" => r(f::_car_named_in_passing(text)),
        "_car" => r(f::_car(text)),
        "_reads_as_a_plural" => r(f::_reads_as_a_plural(a.s(0, "name_word"), a.s(1, "first"))),
        "_place_name" => r(f::_place_name(a.s(0, "phrase"))),
        "_born_place" => r(f::_born_place(text)),
        "_the_person_moved" => r(f::_the_person_moved(a.s(0, "before"))),
        "_from_place" => r(f::_from_place(text)),
        "_places" => r(f::_places(text)),
        "stated_not_asked" => r(f::stated_not_asked(text)),
        "_the_person_is_in_it" => r(f::_the_person_is_in_it(a.s(0, "clause"))),
        "subject" => a.pos_in(text, 1, "pos").and_then(|p| r(f::subject(text, p as i64))),
        "_a_person_word" => r(f::_a_person_word(a.s(0, "word"))),
        "_the_person_without_a_relation" => r(f::_the_person_without_a_relation(a.s(0, "clause"))),
        "_trim_filler" => r(f::_trim_filler(text)),
        "_trim_time_tail" => r(f::_trim_time_tail(a.s(0, "item"))),
        "_a_modifier_before_a_generic_noun" => r(f::_a_modifier_before_a_generic_noun(&a.strs(0, "words"))),
        "_only_stop_words" => r(f::_only_stop_words(a.s(0, "item"))),
        "_allergy_items" => r(f::_allergy_items(a.s(0, "chunk"))),
        "one_more" => r(f::one_more(text)),
        "the_limit_they_share" => r(f::the_limit_they_share(text)),
        "denied_before" => a.pos_in(text, 1, "start").and_then(|s| r(f::denied_before(text, s))),
        "_clause" => a.pos_in(text, 1, "start").and_then(|s| r(f::_clause(text, s))),
        "_the_clause_before" => (|| {
            let edges = a.get(2, "edges").and_then(J::as_i64).unwrap_or(4) as usize;
            r(f::_the_clause_before(text, a.pos_in(text, 1, "start")?, edges))
        })(),
        "_opens_a_subject" => r(f::_opens_a_subject(a.s(0, "clause"))),
        "_around" => a.pos_in(text, 1, "start").and_then(|s| r(f::_around(text, s))),
        "_own_named" => a.pos_in(text, 1, "start").and_then(|s| r(f::_own_named(text, s, &a.v(2, "own")))),
        "_own_in_denial" => r(f::_own_in_denial(text, &a.v(1, "own"))),
        "given_away_one" => r(f::given_away_one_records(text, &a.v(1, "own"))),
        "_pointed_at" => a.pos_in(text, 1, "start").and_then(|s| r(f::_pointed_at(text, s, a.b(2, "plural", false)))),
        "not_had" => a.pos_in(text, 1, "start").and_then(|s| r(f::not_had(text, s))),
        "_have_none" => r(f::_have_none(a.s(0, "clause"))),
        "taken_back_before" => a.pos_in(text, 1, "start").and_then(|s| r(f::taken_back_before(text, s))),
        "denials" => r(f::denials(text, &a.v(1, "own"))),
        "their_back" => r(f::their_back_records(text)),
        "_one_of_several" => r(f::_one_of_several(&a.vals(0, "animals"), a.s(1, "said"), &a.v(2, "own"))),
        "_merge_pets" => r(f::_merge_pets(&a.vals(0, "had"), &a.vals(1, "more"))),
        "_only_pet" => r(f::_only_pet(&a.v(0, "own"))),
        "_names_a_relative" => r(f::_names_a_relative(a.s(0, "said"))),
        "losses" => r(f::losses(text, &a.v(1, "own"), a.s(2, "said_before"))),
        "_allergies" => r(f::_allergies(text)),
        "_month_age_after" => a.pos_in(text, 1, "end").and_then(|e| r(f::_month_age_after(text, e))),
        "_the_word_then_a_verb" => r(f::_the_word_then_a_verb(a.s(0, "name"), a.s(1, "following"))),
        "_may_be_a_name" => r(f::_may_be_a_name(a.s(0, "word"))),
        "a_titled_name" => r(f::a_titled_name(a.s(0, "word"))),
        "_looks_like_a_name" => r(f::_looks_like_a_name(a.s(0, "word"))),
        "_name_after_a_title" => r(f::_name_after_a_title(a.s(0, "name"))),
        "_the_name_without_a_possessive" => {
            let v: Vec<(Value, Value)> =
                f::_the_name_without_a_possessive(text).into_iter().map(|(k, v)| (Value::from(k), Value::from(v))).collect();
            r(Value::Dict(std::sync::Arc::new(v)))
        }
        "_a_relations_name" => r(f::_a_relations_name(text)),
        "figures_as_digits" => r(f::figures_as_digits(text)),
        "kid_ages" => r(f::kid_ages_n(text)),
        "the_ones_are_given_one_at_a_time" => r(f::the_ones_are_given_one_at_a_time(text)),
        "ages_said_one_by_one" => a.n(1, "count").and_then(|c| r(f::ages_said_one_by_one_n(text, c))),
        "kid_count" => r(f::kid_count(text)),
        "count_of_the_people" => r(f::count_of_the_people(text, &a.memory(1, "memory"))),
        "_the_group_in_hand_is_the_children" => r(f::_the_group_in_hand_is_the_children(&a.memory(0, "memory"))),
        "_kids_named_with_ages" => r(f::_kids_named_with_ages(text)),
        "_the_same_person" => r(f::_the_same_person(&a.v(0, "one"), &a.v(1, "other"))),
        "_own_states" => r(f::_own_states(text)),
        "_other_states" => r(f::_other_states(text)),
        "_the_years_are_a_things" => r(f::_the_years_are_a_things(text)),
        "_children_by_year_old" => r(f::_children_by_year_old(text)),
        "_ages_held" => r(f::_ages_held(&a.v(0, "entry"))),
        "_keep_the_ages" => {
            let mut found = a.vals(0, "found");
            f::_keep_the_ages(&mut found, &a.v(1, "entry"));
            Ok(json!({"__after__": Value::from(found).to_plain()}))
        }
        "_other_ages" => r(f::_other_ages(text)),
        "_relation_before" => a.pos_in(text, 1, "pos").and_then(|p| r(f::_relation_before(text, p))),
        "pet_appositive" => {
            let start = a.get(1, "start").map(|_| a.pos_in(text, 1, "start")).unwrap_or(Ok(0));
            start.and_then(|s| r(f::pet_appositive(text, s as i64)))
        }
        "names_offered" => r(f::names_offered(&a.memory(0, "memory"))),
        "a_name_picked" => r(f::a_name_picked(text, &a.memory(1, "memory"))),
        "bare_name" => match a.get(0, "text") {
            Some(J::String(s)) => r(f::bare_name(s)),
            _ => r(Option::<String>::None),
        },
        "is_addressing" => r(f::is_addressing(a.s(0, "word"))),
        "foods_ruled_out" => r(f::foods_ruled_out(&a.v(0, "found"))),
        "_other_no_drink" => r(f::_other_no_drink(text)),
        "_somebody" => r(f::_somebody(a.s(0, "subject_text"))),
        "_other_diets" => r(f::_other_diets(text)),
        "_other_conditions" => r(f::_other_conditions(text)),
        "_the_job_phrase" => r(f::_the_job_phrase(a.s(0, "phrase"), a.b(1, "place_job", false))),
        "_other_jobs" => r(f::_other_jobs(text)),
        "_names_before_a_fact" => r(f::_names_before_a_fact(text).into_iter().map(|(p, n)| (pt::char_index(text, p) as i64, n)).collect::<Vec<_>>()),
        "pet_name_only" => r(f::pet_name_only(text, a.b(1, "lower", false))),
        "_count_before" => a.pos_in(text, 1, "start").and_then(|s| r(f::_count_before(text, s))),
        "pet_counts" => r(f::pet_counts(text)),
        "_names_after" => (|| {
            let most = a.get(2, "most").and_then(J::as_i64);
            r(f::_names_after(a.s(0, "rest"), a.n(1, "count")?, most))
        })(),
        "_not_their_animal" => (|| r(f::_not_their_animal(text, a.pos_in(text, 1, "start")?, a.pos_in(text, 2, "end")?)))(),
        "_someone_elses" => r(f::_someone_elses(text)
            .into_iter()
            .map(|(s, e)| (pt::char_index(text, s) as i64, pt::char_index(text, e) as i64))
            .collect::<Vec<_>>()),
        "_people" => r(f::_people(text)),
        "_a_question" => r(f::_a_question(text)),
        "_owns" => r(f::_owns(text)),
        "_the_thing_of" => r(f::_the_thing_of(a.s(0, "value"))),
        "_the_model" => r(f::_the_model(&a.v(0, "value"), a.s(1, "thing"))),
        "_spell_the_model" => r(f::_spell_the_model(&a.strs(0, "words"))),
        "_a_name_given_by_the_verb" => r(f::_a_name_given_by_the_verb(a.s(0, "word"))),
        "_the_same_animal" => r(f::_the_same_animal(a.s(0, "one"), a.s(1, "other"))),
        "_with_the_breed" => r(f::_with_the_breed(a.v(0, "pet"), a.s(1, "breed"))),
        "_pets" => r(f::_pets(text, a.b(1, "question", false))),
        "_pet_age_tail" => r(f::_pet_age_tail(a.s(0, "after"))),
        "_pet_age" => r(f::_pet_age(a.s(0, "rest"))),
        "_describes_them" => r(f::_describes_them(a.s(0, "word"))),
        "_bare_name" => r(f::_bare_name(a.s(0, "word"), a.b(1, "pair", false))),
        "a_name_could_be" => r(f::a_name_could_be(a.s(0, "word"))),
        "is_a_contraction" => r(f::is_a_contraction(a.s(0, "word"))),
        "_not_a_name" => r(f::_not_a_name(a.s(0, "word"), a.b(1, "strict_ly", true))),
        "_object_words" => r(f::_object_words(a.s(0, "rest"))),
        "_object_plan" => r(f::_object_plan(a.s(0, "rest"))),
        "_a_plan_has_a_thing" => r(f::_a_plan_has_a_thing(a.s(0, "rest"))),
        "_asks_about_the_plan" => a.pos_in(text, 1, "at").and_then(|p| r(f::_asks_about_the_plan(text, p))),
        "_plan_text" => a.pos_in(text, 1, "start").and_then(|s| r(f::_plan_text(text, s))),
        "_how_they_are" => r(f::_how_they_are(a.s(0, "clause"))),
        "_how_their_things_are" => r(f::_how_their_things_are(a.s(0, "clause"))),
        "_the_day_described" => r(f::_the_day_described(a.s(0, "thing"))),
        "_appointment" => r(f::_appointment(text)),
        "_coming_up" => r(f::_coming_up(a.s(0, "clause"))),
        "_dated_plan" => {
            let mut deed = a.strs(1, "deed");
            let given = a.get(1, "deed").is_some_and(|d| d.is_array());
            let got = f::_dated_plan(text, if given { Some(&mut deed) } else { None });
            Ok(json!({"__result__": got, "__deed__": deed}))
        }
        "_the_opener_end" => {
            let said = a.s(0, "said");
            r(pt::char_index(said, f::_the_opener_end(said)) as i64)
        }
        "without_the_correction_opener" => r(f::without_the_correction_opener(text)),
        "is_a_correction_opener" => r(f::is_a_correction_opener(text)),
        "_said_after_the_subject" => r(f::_said_after_the_subject(a.s(0, "clause"))),
        "_the_verb_for" => r(f::_the_verb_for(a.s(0, "body"), a.s(1, "subject"))),
        "_with_the_verb_lent" => r(f::_with_the_verb_lent(a.s(0, "clause"), a.s(1, "earlier"))),
        "_with_the_person_joined_to_the_subject" => {
            r(f::_with_the_person_joined_to_the_subject(a.s(0, "ahead"), a.s(1, "clause")))
        }
        "_continued_parts" => r(f::_continued_parts(text)),
        "_the_relation_named" => r(f::_the_relation_named(a.s(0, "clause"))),
        "_a_clause_about_the_same_person" => r(f::_a_clause_about_the_same_person(text)),
        "_self_intro_parts" => r(f::_self_intro_parts(text)),
        "_same_thing" => r(f::_same_thing(&value_text(a.get(0, "one")), &value_text(a.get(1, "other")))),
        "_is_only_the_take_back" => r(f::_is_only_the_take_back(&a.v(0, "value"), &a.v(1, "gone"))),
        "bot_name" => r(f::bot_name(text)),
        "a_correction_without_a_subject" => r(f::a_correction_without_a_subject(text, a.s(1, "before"))),
        "_reported_clause" => r(f::_reported_clause(text)),
        "_paid_by_a_period" => a.pos_in(text, 1, "end").and_then(|e| r(f::_paid_by_a_period(text, e))),
        "_a_years_pay" => r(f::_a_years_pay(&value_text(a.get(0, "figure")))),
        "extract" => r(f::extract(text, a.b(1, "question", false))),
        "about_themselves" => r(f::about_themselves_facts(a.s(0, "said"))),
        "_one_entry_per_person" => r(f::_one_entry_per_person(&a.vals(0, "entries"))),
        "_a_name_and_a_figure" => r(f::_a_name_and_a_figure(text)),
        "the_part_it_hurts" => r(f::the_part_it_hurts(text)),
        "_extract" => r(f::_extract(text, a.b(1, "question", false))),
        _ => return None,
    })
}

/// `str(x)` for a recorded scalar.
fn value_text(j: Option<&J>) -> String {
    match j {
        Some(J::String(s)) => s.clone(),
        Some(J::Null) | None => "None".into(),
        Some(J::Bool(b)) => if *b { "True".into() } else { "False".into() },
        Some(J::Number(n)) => n.to_string(),
        Some(other) => other.to_string(),
    }
}

/// What Python's result looks like in the shape the checker returns.
fn expected(row: &J, name: &str) -> J {
    let fname = name.rsplit(':').next().unwrap_or(name);
    match fname {
        "_keep_the_ages" => json!({"__after__": row.get("after").and_then(|a| a.get(0)).cloned().unwrap_or_else(|| row["args"][0].clone())}),
        "_dated_plan" => {
            let deed = row.get("after").and_then(|a| a.get(1)).cloned().unwrap_or_else(|| row["args"].get(1).cloned().unwrap_or(J::Null));
            json!({"__result__": row["result"], "__deed__": if deed.is_null() { json!([]) } else { deed }})
        }
        "_correct_count" | "_correct_a_persons_name" | "_a_value_already_the_plans" | "_a_correction_is_not_a_plan"
        | "_correct_their_birthday" | "_a_birthday_for_somebody" | "_the_person_the_plan_is_about"
        | "_correct_plan" | "_correct_fact" | "_correct_date" | "_correct_clock" | "_correct_number" | "_correct_when"
        | "_correct_the_count_on_the_table" | "_correct_other" | "_correct_birthday" | "_a_name_the_store_keeps"
        | "_apply_correction"
            if name.starts_with("bot.memory.corrected") =>
        {
            let mut after = row.get("after").and_then(|a| a.get(0)).cloned().unwrap_or_else(|| row["args"][0].clone());
            if let Some(o) = after.as_object_mut() {
                o.remove("understood");
            }
            json!({"__result__": row["result"], "__memory__": after})
        }
        _ => row["result"].clone(),
    }
}

use rsbot::memory::again as ag;
use rsbot::memory::corrected as co;
use rsbot::memory::rename as rn;

fn call_again(fname: &str, a: &Args) -> Option<Result<J, Skip>> {
    let mem = |i: usize, n: &str| a.memory(i, n);
    Some(match fname {
        "the_name_the_answer_says" => r(ag::the_name_the_answer_says(a.s(0, "answer"), a.s(1, "detail"), a.s(2, "card"))),
        "as_a_phrase" => r(ag::as_a_phrase(a.s(0, "text"))),
        "_a_phrase_of" => r(ag::_a_phrase_of(a.s(0, "text"))),
        "_says_i" => r(ag::_says_i(a.s(0, "text"))),
        "_for_the_person" => r(ag::_for_the_person(a.s(0, "text"))),
        "the_same_thing_underneath" => a.n(2, "times").and_then(|t| r(ag::the_same_thing_underneath(a.s(0, "text"), a.s(1, "reply"), t, &mem(3, "memory")))),
        "unframed" => r(ag::unframed(a.s(0, "reply"))),
        "_flat" => r(ag::_flat(a.s(0, "text"))),
        "the_cause_said_again" => r(ag::the_cause_said_again(a.s(0, "reply"), &mem(1, "memory"))),
        "_a_card_behind" => r(ag::_a_card_behind(&mem(0, "memory"))),
        "_led_an_earlier_answer" => r(ag::_led_an_earlier_answer(a.s(0, "sentence"), &mem(1, "memory"))),
        "_the_answer_in_hand" => r(ag::_the_answer_in_hand(a.s(0, "reply"), &mem(1, "memory"))),
        "_short" => {
            let m = a.get(1, "memory").filter(|j| j.is_object()).map(|_| mem(1, "memory"));
            r(ag::_short(a.s(0, "reply"), m.as_ref()))
        }
        "_the_first_sentence" => r(ag::_the_first_sentence(a.s(0, "reply"), &mem(1, "memory"))),
        "_carries_a_lead_in" => r(ag::_carries_a_lead_in(a.s(0, "mine"))),
        "carries_a_lead_in" => r(ag::carries_a_lead_in(a.s(0, "reply"))),
        "_times_on_the_screen" => r(ag::_times_on_the_screen(a.s(0, "mine"), &mem(1, "memory"))),
        "_said_earlier" => r(ag::_said_earlier(a.s(0, "mine"), &mem(1, "memory"))),
        "_said_earlier_text" => r(ag::_said_earlier_text(a.s(0, "mine"), &mem(1, "memory"))),
        "_names_something_the_answer_says" => r(ag::_names_something_the_answer_says(a.s(0, "text"), a.s(1, "mine"))),
        "_returned_thing" => r(ag::_returned_thing(&mem(0, "memory"), a.s(1, "text"))),
        "_the_line_asks" => r(ag::_the_line_asks(&mem(0, "memory"), a.s(1, "text"))),
        "_adds_nothing_the_answer_did_not_say" => {
            r(ag::_adds_nothing_the_answer_did_not_say(a.s(0, "thing"), a.s(1, "text"), a.s(2, "said"), &mem(3, "memory")))
        }
        "_as_a_name" => r(ag::_as_a_name(a.s(0, "thing"))),
        "_without_the_thing_in_front" => r(ag::_without_the_thing_in_front(a.s(0, "short"), a.s(1, "thing"))),
        "_said_recently" => r(ag::_said_recently(a.s(0, "mine"), &mem(1, "memory"), a.get(2, "look").and_then(J::as_i64).unwrap_or(3))),
        "_says_it_came_back" => r(ag::_says_it_came_back(a.s(0, "text"))),
        "_a_figure_asked_that_the_answer_does_not_hold" => r(ag::_a_figure_asked_that_the_answer_does_not_hold(a.s(0, "text"), a.s(1, "reply"))),
        "_asks_for_a_detail" => r(ag::_asks_for_a_detail(a.s(0, "text"))),
        "_a_report" => r(ag::_a_report(a.s(0, "text"))),
        "_has_a_script" => r(ag::_has_a_script(a.s(0, "reply"))),
        "_the_line_that_got_it" => r(ag::_the_line_that_got_it(a.s(0, "mine"), &mem(1, "memory"), a.get(2, "look").and_then(J::as_i64).unwrap_or(3))),
        "_said_and_more" => r(ag::_said_and_more(a.s(0, "mine"), &mem(1, "memory"))),
        "already_had" => r(ag::already_had(a.s(0, "reply"), &mem(1, "memory"))),
        "_as_said" => r(ag::_as_said(a.s(0, "text"))),
        "_content_words_in_order" => r(ag::_content_words_in_order(a.s(0, "text"))),
        "_the_run_they_said_both_times" => {
            let earlier: std::collections::HashSet<String> = a.strs(1, "earlier").into_iter().collect();
            r(ag::_the_run_they_said_both_times(a.s(0, "text"), &earlier))
        }
        "_the_same_question_asked_again" => r(ag::_the_same_question_asked_again(a.s(0, "mine"), a.s(1, "text"), &mem(2, "memory"))),
        "_the_subject_on_the_screen" => r(ag::_the_subject_on_the_screen(&mem(0, "memory"))),
        "_as_your" => r(ag::_as_your(a.s(0, "subject"))),
        "instead" => r(ag::instead(a.s(0, "reply"), a.s(1, "text"), &mem(2, "memory"))),
        "a_thing_of_the_answer" => r(ag::a_thing_of_the_answer(a.s(0, "reply"), a.s(1, "text"), &mem(2, "memory"))),
        "_another_stand_down" => r(ag::_another_stand_down(a.s(0, "reply"), &mem(1, "memory"))),
        "_framed_as_a_repeat" => a.n(3, "times").and_then(|t| r(ag::_framed_as_a_repeat(a.s(0, "reply"), a.s(1, "mine"), &mem(2, "memory"), t, a.s(4, "text")))),
        _ => return None,
    })
}

/// The memory after a call, as the recording keeps it (the reading itself left out: it is typed
/// in the port and plain in Python, and none of these readers writes it).
fn memory_plain(m: &Memory) -> J {
    let mut map = m.to_plain();
    map.remove("understood");
    J::Object(map)
}

/// A reader that writes memory: its result and the memory after it.
fn with_memory<T: ToJ>(a: &Args, f: impl FnOnce(&mut Memory) -> T) -> Result<J, Skip> {
    let mut m = a.memory(0, "memory");
    let got = f(&mut m).j();
    Ok(json!({"__result__": got, "__memory__": memory_plain(&m)}))
}

fn call_corrected(fname: &str, a: &Args) -> Option<Result<J, Skip>> {
    let mem = || a.memory(0, "memory");
    match fname {
        "_correct_count" => return Some(with_memory(a, |m| co::_correct_count(m, &a.v(1, "target"), a.s(2, "text")))),
        "_correct_a_persons_name" => {
            return Some(with_memory(a, |m| co::_correct_a_persons_name(m, &a.v(1, "target"), a.s(2, "text"), &a.v(3, "wrong"))))
        }
        "_a_value_already_the_plans" => return Some(with_memory(a, |m| co::_a_value_already_the_plans(m, &a.v(1, "target"), a.s(2, "text")))),
        "_a_correction_is_not_a_plan" => return Some(with_memory(a, |m| { co::_a_correction_is_not_a_plan(m, a.s(1, "text")); Value::None })),
        "_correct_their_birthday" => {
            return Some(with_memory(a, |m| co::_correct_their_birthday(m, &a.v(1, "target"), &a.v(2, "wrong"), a.s(3, "text"))))
        }
        "_a_birthday_for_somebody" => return Some(with_memory(a, |m| co::_a_birthday_for_somebody(m))),
        "_correct_plan" => return Some(with_memory(a, |m| co::_correct_plan(m, &a.v(1, "target"), &a.v(2, "wrong"), a.s(3, "text")))),
        "_correct_fact" => return Some(with_memory(a, |m| co::_correct_fact(m, &a.v(1, "target"), &a.v(2, "wrong"), a.s(3, "text")))),
        "_correct_date" => return Some(with_memory(a, |m| co::_correct_date(m, &a.v(1, "target"), a.s(2, "text"), &a.v(3, "wrong")))),
        "_correct_clock" => return Some(with_memory(a, |m| co::_correct_clock(m, &a.v(1, "target"), a.s(2, "text"), &a.v(3, "wrong")))),
        "_correct_number" => return Some(with_memory(a, |m| co::_correct_number(m, &a.v(1, "target"), a.s(2, "text")))),
        "_correct_when" => return Some(with_memory(a, |m| co::_correct_when(m, &a.v(1, "target"), a.s(2, "text")))),
        "_correct_the_count_on_the_table" => {
            return Some(with_memory(a, |m| co::_correct_the_count_on_the_table(m, &a.v(1, "target"), a.s(2, "text"))))
        }
        "_correct_other" => return Some(with_memory(a, |m| co::_correct_other(m, &a.v(1, "target"), a.s(2, "text")))),
        "_correct_birthday" => return Some(with_memory(a, |m| co::_correct_birthday(m, &a.v(1, "target"), a.s(2, "text")))),
        "_a_name_the_store_keeps" => return Some(with_memory(a, |m| co::_a_name_the_store_keeps(m, a.s(1, "taken_back"), a.s(2, "new")))),
        "_apply_correction" => return Some(with_memory(a, |m| { co::_apply_correction(m, &a.v(1, "correction"), a.s(2, "text")); Value::None })),
        "_the_person_the_plan_is_about" => {
            let before = a.get(2, "before").and_then(J::as_str).map(str::to_string);
            return Some(with_memory(a, |m| co::_the_person_the_plan_is_about(m, &a.v(1, "correction"), before.as_deref())));
        }
        "_stated_last" => return Some(r(co::_stated_last(&mem()))),
        "_stated_plan" => return Some(r(co::_stated_plan(&mem()))),
        "_fact_naming" => return Some(r(co::_fact_naming(&mem(), &a.v(1, "wrong")))),
        "_a_birthday_on_the_last_line" => return Some(r(co::_a_birthday_on_the_last_line(&mem()))),
        "_number_belongs_to_that_age" => return Some(r(co::_number_belongs_to_that_age(&mem()))),
        "_the_plan_on_the_table" => return Some(r(co::_the_plan_on_the_table(&mem()))),
        "_make_corrects" => return Some(r(co::_make_corrects(&a.v(0, "target"), &a.memory(1, "memory")))),
        _ => {}
    }
    Some(match fname {
        "_a_name_for_a_person" => r(co::_a_name_for_a_person(&a.v(0, "value"))),
        "_a_new_name_was_said" => r(co::_a_new_name_was_said(&a.v(0, "target"), a.s(1, "text"))),
        "_a_word_of_a_name" => r(co::_a_word_of_a_name(a.s(0, "word"))),
        "_denies_and_names" => r(co::_denies_and_names(a.s(0, "text"))),
        "_the_unit" => r(co::_the_unit(a.s(0, "text"))),
        "_measured_the_same" => r(co::_measured_the_same(&a.v(0, "value"), a.s(1, "before"))),
        "_without_the_naming_verb" => r(co::_without_the_naming_verb(&a.v(0, "value"))),
        "_what_took_its_place" => r(co::_what_took_its_place(a.s(0, "before"), a.s(1, "after"), a.s(2, "value"))),
        "_a_day_written_the_way_the_store_writes_it" => r(co::_a_day_written_the_way_the_store_writes_it(a.s(0, "fact"))),
        "_an_amount_on_the_measure" => r(co::_an_amount_on_the_measure(a.s(0, "plan"), &a.v(1, "target"))),
        "_number_in" => r(co::_number_in(&a.v(0, "value"))),
        "_a_spelling_of" => r(co::_a_spelling_of(a.s(0, "new"), a.s(1, "old"))),
        "_letters_apart" => r(co::_letters_apart(a.s(0, "one"), a.s(1, "other"))),
        "_the_topic_with_the_new_clock" => r(co::_the_topic_with_the_new_clock(a.s(0, "text"), a.s(1, "plan_now"))),
        "_a_bare_hour" => r(co::_a_bare_hour(&a.v(0, "value"))),
        "_names_a_half_of_the_day" => r(co::_names_a_half_of_the_day(a.s(0, "clock"))),
        "_clock_key" => match a.get(0, "clock") {
            Some(J::String(c)) => r(co::_clock_key(c)),
            _ => Err(Skip::Shape),
        },
        "_count_said" => r(co::_count_said(a.s(0, "text"))),
        "_a_count" => a.n(0, "count").and_then(|c| r(co::_a_count(c, &a.v(1, "animal")))),
        "_date_said" => r(co::_date_said(a.s(0, "low"))),
        "_a_day_in_the_kind" => r(co::_a_day_in_the_kind(a.s(0, "old_day"), a.s(1, "day"))),
        "_date_word_case" => r(co::_date_word_case(&a.v(0, "value"))),
        "_date_in_plan" => r(co::_date_in_plan(a.s(0, "plan"))),
        "_names_a_date" => r(co::_names_a_date(&a.v(0, "value"))),
        "_date_word" => r(co::_date_word(a.s(0, "phrase"))),
        "_kind_of_value" => r(co::_kind_of_value(&a.v(0, "target"), &a.v(1, "told"))),
        "_swap_in" => r(co::_swap_in(&a.v(0, "told"), &a.v(1, "target"), &a.v(2, "wrong"))),
        "_carries" => r(co::_carries(&a.v(0, "value"), a.s(1, "word"))),
        "_the_line_is_about_that_birthday" => r(co::_the_line_is_about_that_birthday(a.s(0, "text"), &a.v(1, "held"), &a.v(2, "target"))),
        "_keep_unit" => r(co::_keep_unit(&a.v(0, "value"), &a.v(1, "was"))),
        "_birthday_words" => r(co::_birthday_words(&a.v(0, "date"))),
        "_many" => a.n(0, "count").and_then(|c| r(co::_many(c, &a.v(1, "animal")))),
        "_asks" => r(co::_asks(a.s(0, "line"))),
        _ => return None,
    })
}

fn call_rename(fname: &str, a: &Args) -> Option<Result<J, Skip>> {
    Some(match fname {
        "moves" => r(rn::moves(a.s(0, "low"))),
        "the_name_it_gave" => r(rn::the_name_it_gave(a.s(0, "low"))),
        "the_name_a_line_moved_on" => r(rn::the_name_a_line_moved_on(a.s(0, "low"), &a.v(1, "pets"), &a.v(2, "animal_named"))),
        _ => return None,
    })
}


fn main() {
    rsbot::runtime::quiet_panics();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = args.first().expect("a recording from harness/mem2_record.py");
    let (mut fns, mut show) = (Vec::new(), 3usize);
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--fn" => {
                fns.push(args[i + 1].clone());
                i += 1;
            }
            "--show" => {
                show = args[i + 1].parse().unwrap();
                i += 1;
            }
            x => panic!("unknown argument {x}"),
        }
        i += 1;
    }
    let file = std::fs::File::open(path).unwrap();
    let reader = std::io::BufReader::new(flate2::read::GzDecoder::new(file));
    // fn -> [same, differ, unported, shape, no port]
    let mut tally: BTreeMap<String, [usize; 5]> = BTreeMap::new();
    let mut shown: BTreeMap<String, usize> = BTreeMap::new();
    let empty = serde_json::Map::new();
    for line in reader.lines() {
        let row: J = serde_json::from_str(&line.unwrap()).unwrap();
        let name = row["fn"].as_str().unwrap().to_string();
        if !fns.is_empty() && !fns.iter().any(|f| name.contains(f.as_str())) {
            continue;
        }
        let t = tally.entry(name.clone()).or_default();
        if row["raised"].is_string() || has_tag(&row["args"]) || has_tag(&row["kwargs"]) {
            t[3] += 1;
            continue;
        }
        let pos = row["args"].as_array().cloned().unwrap_or_default();
        let kw = row["kwargs"].as_object().unwrap_or(&empty);
        let a = Args { pos: &pos, kw };
        let got = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| call(&name, &a)));
        match got {
            Ok(None) => t[4] += 1,
            Ok(Some(Err(Skip::Shape))) => t[3] += 1,
            Ok(Some(Ok(j))) => {
                let want = expected(&row, &name);
                if j == want {
                    t[0] += 1;
                } else {
                    t[1] += 1;
                    let n = shown.entry(name.clone()).or_default();
                    if *n < show {
                        *n += 1;
                        println!("DIFF {name} args={}\n  python: {}\n  rust:   {}", row["args"], want, j);
                    }
                }
            }
            Err(e) => {
                let msg = e
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_default();
                if msg.contains("UNPORTED") {
                    t[2] += 1;
                } else {
                    t[1] += 1;
                    let n = shown.entry(name.clone()).or_default();
                    if *n < show {
                        *n += 1;
                        println!("PANIC {name} args={}\n  {msg}", row["args"]);
                    }
                }
            }
        }
    }
    let mut total = [0usize; 5];
    println!("\n{:<58} {:>6} {:>6} {:>8} {:>6} {:>6}", "function", "same", "differ", "unported", "shape", "noport");
    for (name, t) in &tally {
        for k in 0..5 {
            total[k] += t[k];
        }
        println!("{:<58} {:>6} {:>6} {:>8} {:>6} {:>6}", name, t[0], t[1], t[2], t[3], t[4]);
    }
    println!("{:<58} {:>6} {:>6} {:>8} {:>6} {:>6}", "TOTAL", total[0], total[1], total[2], total[3], total[4]);
}
