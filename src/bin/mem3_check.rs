//! Workstream MEM3's function-level check: replay the calls `harness/mem3_calls.py` recorded
//! while PyBot ran whole conversations, through the Rust port of bot/memory/topics.py, named.py,
//! thread.py and deadline.py, and compare the results.
//!
//!     mem3_check CALLS.jsonl.gz [--show N] [--only FN]
use flate2::read::GzDecoder;
use rsbot::data::Value;
use rsbot::knowledge::entry::Entry;
use rsbot::memory::{deadline, named, thread, topics};
use rsbot::state::Memory;
use rsbot::understood::Understood;
use serde_json::{json, Value as J};
use std::collections::{BTreeMap, HashSet};
use std::io::BufRead;

struct Args {
    args: Vec<J>,
    kwargs: serde_json::Map<String, J>,
    alias: Option<usize>,
}

impl Args {
    fn j(&self, i: usize, name: &str) -> J {
        self.args.get(i).cloned().or_else(|| self.kwargs.get(name).cloned()).unwrap_or(J::Null)
    }
    fn has(&self, i: usize, name: &str) -> bool {
        self.args.len() > i || self.kwargs.contains_key(name)
    }
    fn s(&self, i: usize, name: &str) -> String {
        match self.j(i, name) {
            J::String(s) => s,
            J::Null => String::new(),
            other => other.to_string(),
        }
    }
    fn mem(&self, i: usize, name: &str) -> Memory {
        let m = match self.j(i, name) {
            J::Object(m) => Memory::from_plain(&m),
            _ => Memory::new(),
        };
        let mut m = m;
        if let Some(k) = self.alias {
            if let Some(topic) = m.get("topic").cloned() {
                if let Some(list) = m.get_mut("topics").and_then(Value::items_mut) {
                    if k < list.len() {
                        list[k] = topic;
                    }
                }
            }
        }
        m
    }
    fn und(&self, i: usize, name: &str) -> Understood {
        serde_json::from_value(self.j(i, name)).unwrap_or_default()
    }
    fn und_opt(&self, i: usize, name: &str) -> Option<Understood> {
        match self.j(i, name) {
            J::Object(_) => Some(self.und(i, name)),
            _ => None,
        }
    }
    fn val(&self, i: usize, name: &str) -> Value {
        Value::from_plain(&self.j(i, name))
    }
    fn strs(&self, i: usize, name: &str) -> Vec<String> {
        match self.j(i, name) {
            J::Array(a) => a.iter().map(|x| x.as_str().map(str::to_string).unwrap_or_else(|| x.to_string())).collect(),
            _ => Vec::new(),
        }
    }
    fn int(&self, i: usize, name: &str, default: i64) -> i64 {
        self.j(i, name).as_i64().unwrap_or(default)
    }
    fn b(&self, i: usize, name: &str, default: bool) -> bool {
        self.j(i, name).as_bool().unwrap_or(default)
    }
    fn entry(&self, i: usize, name: &str) -> Option<&'static Entry> {
        let j = self.j(i, name);
        j.get("entry").and_then(J::as_str).and_then(rsbot::knowledge::entry::by_id)
    }
}

/// A Rust result as the plain JSON PyBot's result was recorded in.
trait Py {
    fn py(self) -> J;
}
impl Py for String {
    fn py(self) -> J {
        J::String(self)
    }
}
impl Py for &str {
    fn py(self) -> J {
        J::String(self.to_string())
    }
}
impl Py for bool {
    fn py(self) -> J {
        J::Bool(self)
    }
}
impl Py for i64 {
    fn py(self) -> J {
        json!(self)
    }
}
impl Py for usize {
    fn py(self) -> J {
        json!(self)
    }
}
impl Py for f64 {
    fn py(self) -> J {
        json!(self)
    }
}
impl Py for () {
    fn py(self) -> J {
        J::Null
    }
}
impl Py for Value {
    fn py(self) -> J {
        self.to_plain()
    }
}
impl Py for Vec<String> {
    fn py(self) -> J {
        json!(self)
    }
}
impl Py for HashSet<String> {
    fn py(self) -> J {
        let mut v: Vec<String> = self.into_iter().collect();
        v.sort();
        json!(v)
    }
}
impl Py for Understood {
    fn py(self) -> J {
        serde_json::to_value(self).unwrap()
    }
}
impl Py for &'static Entry {
    fn py(self) -> J {
        json!({"entry": self.id})
    }
}
impl<T: Py> Py for Option<T> {
    fn py(self) -> J {
        match self {
            Some(x) => x.py(),
            None => J::Null,
        }
    }
}
impl<A: Py, B: Py> Py for (A, B) {
    fn py(self) -> J {
        J::Array(vec![self.0.py(), self.1.py()])
    }
}

const READING_KEYS: [&str; 6] = ["clean", "words", "norm_text", "text", "kind", "intent"];
const MEMORY_KEYS: [&str; 10] =
    ["topic", "topics", "person", "returned", "return_to", "entry_answer", "entry_answers", "words_this_turn", "topic_this_turn", "understood"];

fn reading_view(j: &J) -> J {
    match j {
        J::Object(m) => J::Object(READING_KEYS.iter().filter_map(|k| m.get(*k).map(|v| (k.to_string(), v.clone()))).collect()),
        other => other.clone(),
    }
}

fn memory_view(j: &J) -> J {
    match j {
        J::Object(m) => J::Object(
            MEMORY_KEYS
                .iter()
                .filter_map(|k| m.get(*k).map(|v| (k.to_string(), if *k == "understood" { reading_view(v) } else { v.clone() })))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// Python's None and "" read the same for a str-or-None result; a set compares sorted.
fn norm(j: &J) -> J {
    match j {
        J::String(s) if s.is_empty() => J::Null,
        J::Array(a) if a.is_empty() => J::Array(vec![]),
        other => other.clone(),
    }
}

/// Call the Rust port on the recorded arguments: (result, memory after) or None when this
/// function is not replayed here.
fn call(name: &str, a: &Args) -> Option<(J, Option<J>)> {
    let (module, fname) = name.split_once(':').unwrap();
    let r: J = match (module, fname) {
        ("bot.memory.deadline", "the_deadline") => deadline::the_deadline(&a.s(0, "text")).py(),
        ("bot.memory.deadline", "_a_plan") => deadline::_a_plan(&a.s(0, "said")).py(),
        ("bot.memory.deadline", "follow_up") => deadline::follow_up(&a.s(0, "text"), &a.mem(1, "memory")).py(),
        ("bot.memory.topics", f) => match f {
            "words" => topics::words(&a.s(0, "text")).py(),
            "stem" => topics::stem(&a.s(0, "word")).py(),
            "keywords" => topics::keywords(&a.s(0, "text")).py(),
            "forms" => topics::forms(&a.s(0, "word")).py(),
            "content" => topics::content(&a.s(0, "text")).py(),
            "_what_they_are_not_doing" => topics::_what_they_are_not_doing(&a.s(0, "text")).into_iter().collect::<HashSet<_>>().py(),
            "is_ack" => topics::is_ack(&a.s(0, "text")).py(),
            "is_small_talk" => topics::is_small_talk(&a.s(0, "text"), &a.s(1, "kind")).py(),
            "_a_place_the_line_moves" => topics::_a_place_the_line_moves(&a.s(0, "text")).py(),
            "_asks_about_the_bot" => topics::_asks_about_the_bot(a.und_opt(0, "read").as_ref()).py(),
            "_note" | "note" => {
                let mut m = a.mem(0, "memory");
                let r = if f == "note" {
                    topics::note(&mut m, &a.s(1, "text"), &a.s(2, "kind"))
                } else {
                    topics::_note(&mut m, &a.s(1, "text"), &a.s(2, "kind"))
                };
                return Some((r.py(), Some(J::Object(m.to_plain()))));
            }
            "_the_record_they_came_back_to" => {
                let m = a.mem(1, "memory");
                let rec = m.get("topic").cloned().filter(|t| *t == a.val(2, "record")).unwrap_or_else(|| a.val(2, "record"));
                topics::_the_record_they_came_back_to(&a.s(0, "text"), &m, &rec).py()
            }
            "_the_aim_without_its_deadline" => topics::_the_aim_without_its_deadline(&a.s(0, "text")).py(),
            "_asks_which_kind" => topics::_asks_which_kind(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_asks_an_aspect_of_the_thing" => topics::_asks_an_aspect_of_the_thing(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_reaches_another_thing" => topics::_reaches_another_thing(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_names_a_thing_of_its_own" => {
                let m = a.mem(2, "memory");
                topics::_names_a_thing_of_its_own(&a.s(0, "text"), &a.val(1, "record"), a.has(2, "memory").then_some(&m)).py()
            }
            "_names_a_place_of_its_own" => {
                let m = a.mem(2, "memory");
                topics::_names_a_place_of_its_own(&a.s(0, "text"), &a.val(1, "record"), a.has(2, "memory").then_some(&m)).py()
            }
            "_only_adds_a_person_to_the_thing" => topics::_only_adds_a_person_to_the_thing(&a.s(0, "text"), &a.val(1, "record")).py(),
            "_another_area" => topics::_another_area(&a.s(0, "text"), &a.s(1, "said")).py(),
            "_a_detail_of_what_we_are_on" => topics::_a_detail_of_what_we_are_on(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_a_part_of_the_record" => topics::_a_part_of_the_record(a.und_opt(0, "read").as_ref(), &a.val(1, "record")).py(),
            "_names_another_subject" => topics::_names_another_subject(&a.s(0, "text"), &a.val(1, "record"), &a.mem(2, "memory")).py(),
            "_leans_back" => topics::_leans_back(&a.s(0, "text"), &a.val(1, "record")).py(),
            "_points_at_the_record" => topics::_points_at_the_record(&a.s(0, "text"), &a.val(1, "record"), &a.mem(2, "memory")).py(),
            "_leads_a_question" => topics::_leads_a_question(&a.s(0, "text")).py(),
            "_switches" => topics::_switches(&a.s(0, "text")).py(),
            "opens_by_switching" => topics::opens_by_switching(&a.s(0, "text")).py(),
            "_without_switchers" => topics::_without_switchers(&a.s(0, "text")).py(),
            "_only_details" => topics::_only_details(&a.s(0, "text"), &a.val(1, "record")).py(),
            "_the_plan_a_correction_renames" => {
                let mut m = a.mem(0, "memory");
                let r = topics::_the_plan_a_correction_renames(&mut m, &a.s(1, "text"));
                return Some((r.py(), Some(J::Object(m.to_plain()))));
            }
            "_is_a_quantity" => topics::_is_a_quantity(&a.und(0, "reading")).py(),
            "_is_a_when_or_how_often" => topics::_is_a_when_or_how_often(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_only_names_the_constraint" => topics::_only_names_the_constraint(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_adds_a_detail" => topics::_adds_a_detail(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "person" => topics::person(&a.s(0, "text")).py(),
            "_named_inside_a_question" => topics::_named_inside_a_question(&a.s(0, "text")).py(),
            "person_named" => topics::person_named(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "about" => topics::about(&a.mem(0, "memory")).py(),
            "trouble" => topics::trouble(&a.s(0, "text")).py(),
            "_keep_person" => {
                let mut m = a.mem(0, "memory");
                let r = topics::_keep_person(&mut m, &a.s(1, "text"));
                return Some((r.py(), Some(J::Object(m.to_plain()))));
            }
            "_after_an_emergency" => topics::_after_an_emergency(&a.mem(0, "memory")).py(),
            "_after_a_correction" => topics::_after_a_correction(&a.mem(0, "memory")).py(),
            "in_a_safety_thread" => topics::in_a_safety_thread(&a.mem(0, "memory")).py(),
            "_the_apostrophe_back" => topics::_the_apostrophe_back(&a.s(0, "low")).py(),
            "return_target" => topics::return_target(&a.s(0, "text")).py(),
            "_a_question_after_the_thing" => topics::_a_question_after_the_thing(&a.s(0, "rest"), a.int(1, "split", 0) as usize).py(),
            "_the_thing" => topics::_the_thing(&a.s(0, "rest")).py(),
            "_asks" => topics::_asks(&a.s(0, "text")).py(),
            "asks_after" => topics::asks_after(&a.s(0, "line"), &a.s(1, "thing")).py(),
            "the_thing_asked_again" => topics::the_thing_asked_again(&a.mem(0, "memory"), &a.s(1, "text")).py(),
            "the_thing_went_back_to" => topics::the_thing_went_back_to(&a.mem(0, "memory")).py(),
            "resumed" => topics::resumed(&a.mem(0, "memory")).py(),
            "current" => topics::current(&a.mem(0, "memory")).cloned().py(),
            "alike" => topics::alike(&a.s(0, "one"), &a.s(1, "other")).py(),
            "find" => topics::find(&a.mem(0, "memory"), &a.s(1, "text")).cloned().py(),
            "is_follow_up" => topics::is_follow_up(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "context" => topics::context(&a.mem(0, "memory"), a.int(1, "n", 2)).py(),
            "expand" => topics::expand(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_gerund" => topics::_gerund(&a.s(0, "verb")).py(),
            "_strip_considering" => topics::_strip_considering(&a.s(0, "source")).py(),
            "_only_a_handoff" => topics::_only_a_handoff(&a.s(0, "head")).py(),
            "_a_word_of_a_date" => topics::_a_word_of_a_date(&a.s(0, "word")).py(),
            "_only_a_date" => topics::_only_a_date(&a.s(0, "clause")).py(),
            "_the_thing_after_a_date" => topics::_the_thing_after_a_date(&a.s(0, "text")).py(),
            "gist" => topics::gist(&a.s(0, "text")).py(),
            "_room_limit" => topics::_room_limit(&a.s(0, "text")).py(),
            "_about_record" => topics::_about_record(&a.mem(0, "memory")).py(),
            "_names_nothing" => topics::_names_nothing(&a.val(0, "record")).py(),
            "subject" => topics::subject(&a.mem(0, "memory")).py(),
            "topic_text" => topics::topic_text(&a.mem(0, "memory")).py(),
            "action" => topics::action(&a.mem(0, "memory")).py(),
            "written_out" => topics::written_out(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_named_a_thing" => topics::_named_a_thing(&a.s(0, "line")).py(),
            "_the_thing_on_the_table" => topics::_the_thing_on_the_table(&a.mem(0, "memory")).py(),
            "attribute_of_the_thing" => topics::attribute_of_the_thing(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "key_subject" => {
                let m = a.mem(1, "memory");
                topics::key_subject(&a.s(0, "text"), a.j(1, "memory").is_object().then_some(&m)).py()
            }
            "_the_part_a_complaint_is_about" => topics::_the_part_a_complaint_is_about(&a.s(0, "text")).py(),
            "_the_cause_words" => topics::_the_cause_words(&a.s(0, "text")).py(),
            "_subject_words_to_carry" => topics::_subject_words_to_carry(&a.val(0, "record"), &a.s(1, "text"), &a.mem(2, "memory")).py(),
            "_where_the_topic_lands" => topics::_where_the_topic_lands(&a.s(0, "topic")).py(),
            "an_entry_for" => topics::_entry(&a.s(0, "text")).py(),
            "_entry" => topics::_entry(&a.s(0, "text")).py(),
            "_the_last_question" => topics::_the_last_question(&a.mem(0, "memory")).py(),
            "how_much_of_the_thing_in_hand" => topics::how_much_of_the_thing_in_hand(&a.s(0, "line"), &a.mem(1, "memory")).py(),
            "elided_ask" => topics::elided_ask(&a.s(0, "text")).py(),
            "names_nothing" => topics::names_nothing(&a.s(0, "text")).py(),
            "_content_words" => topics::_content_words(&a.s(0, "text")).py(),
            "_the_topic_phrase" => topics::_the_topic_phrase(&a.s(0, "topic")).py(),
            "the_city_of" => topics::the_city_of(&a.s(0, "topic")).py(),
            "the_country_of" => topics::the_country_of(&a.s(0, "topic")).py(),
            "asks_how_long_to_stay" => topics::asks_how_long_to_stay(&a.s(0, "line")).py(),
            "going_there" => topics::going_there(&a.s(0, "place"), &a.s(1, "topic")).py(),
            "a_stay_with_the_place" => topics::a_stay_with_the_place(&a.s(0, "line"), &a.s(1, "topic")).py(),
            "_the_last_answer" => topics::_the_last_answer(&a.mem(0, "memory")).py(),
            "_the_topics_on_the_chat" => {
                let m = a.mem(1, "memory");
                topics::_the_topics_on_the_chat(&a.s(0, "topic"), a.j(1, "memory").is_object().then_some(&m)).py()
            }
            "_the_thing_a_measure_is_of" => {
                let m = a.mem(2, "memory");
                topics::_the_thing_a_measure_is_of(&a.s(0, "line"), &a.s(1, "topic"), a.j(2, "memory").is_object().then_some(&m)).py()
            }
            "_a_measure_of_the_topic" => {
                let m = a.mem(2, "memory");
                topics::_a_measure_of_the_topic(&a.s(0, "line"), &a.s(1, "topic"), a.j(2, "memory").is_object().then_some(&m)).py()
            }
            "_with_the_word_before" => topics::_with_the_word_before(&a.s(0, "line"), &a.s(1, "word"), &a.s(2, "thing")).py(),
            "_how_they_feel" => topics::_how_they_feel(&a.s(0, "word")).py(),
            "carry_the_ask" => {
                let m = a.mem(3, "memory");
                let u = a.und_opt(2, "understood");
                topics::carry_the_ask_with(&a.s(0, "line"), &a.s(1, "topic"), u.as_ref(), a.j(3, "memory").is_object().then_some(&m)).py()
            }
            "_with_the_article" => topics::_with_the_article(&a.s(0, "line")).py(),
            "_only_said_as_a_frequency" => topics::_only_said_as_a_frequency(&a.s(0, "topic")).py(),
            "_asks_a_property_and_nothing_else" => topics::_asks_a_property_and_nothing_else(&a.s(0, "text")).py(),
            "_carry_the_words" => topics::_carry_the_words(&a.s(0, "line"), &a.s(1, "topic")).py(),
            "the_thing_the_subject_brings" => topics::the_thing_the_subject_brings(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "follows_on" => topics::follows_on(&a.und(0, "understood"), &a.s(1, "text")).py(),
            "keeps_away" => topics::keeps_away_from(&a.s(0, "line"), &a.s(1, "topic"), a.entry(2, "item")).py(),
            "_kept_away_from" => topics::_kept_away_from(&a.s(0, "phrase"), &a.strs(1, "said")).py(),
            "_names_it" => topics::_names_it(&a.strs(0, "found"), a.entry(1, "item")?).py(),
            "_key_words_of" => topics::_key_words_of(a.entry(0, "item")?).py(),
            "_names_it_as_written" => topics::_names_it_as_written(&a.strs(0, "found"), a.entry(1, "item")?).py(),
            "_asked_only" => topics::_asked_only(&a.strs(0, "found"), a.entry(1, "item")?).py(),
            "_a_sky_ask" => topics::_a_sky_ask(&a.s(0, "line")).py(),
            "_the_step_they_report" => topics::_the_step_they_report(&a.s(0, "line")).py(),
            "widen" => topics::widen(&a.s(0, "line"), &a.strs(1, "subject_words"), &a.s(2, "topic_text")).py(),
            "adds_only_a_constraint" => topics::adds_only_a_constraint(&a.s(0, "line")).py(),
            "_constrains_the_ask" => topics::_constrains_the_ask(&a.s(0, "line"), &a.s(1, "topic_text")).py(),
            "_clauses_of" => topics::_clauses_of(&a.s(0, "sentence")).py(),
            "remember_answer" => {
                let mut m = a.mem(0, "memory");
                topics::remember_answer(&mut m, &a.s(1, "reply"), &a.strs(2, "keys"));
                return Some((J::Null, Some(J::Object(m.to_plain()))));
            }
            "entry_answer" => topics::entry_answer(&a.mem(0, "memory"), &a.s(1, "topic_text")).py(),
            "answer_in_hand" => topics::answer_in_hand(&a.mem(0, "memory")).py(),
            "newest_answer" => topics::newest_answer(&a.mem(0, "memory")).py(),
            "_carries" => topics::_carries(&a.s(0, "said"), &a.s(1, "mine")).py(),
            "_only_a_close" => topics::_only_a_close(&a.s(0, "text")).py(),
            "only_acknowledges" => topics::only_acknowledges(&a.s(0, "said")).py(),
            "_the_opening_of" => topics::_the_opening_of(&a.s(0, "reply")).py(),
            "said_again" => {
                let line = a.j(2, "line");
                let line = line.as_str();
                topics::said_again_on(&a.s(0, "reply"), &a.mem(1, "memory"), line).py()
            }
            "_said_later_than" => topics::_said_later_than(&a.val(0, "turn"), &a.val(1, "record")).py(),
            "_the_turn_we_lean_on" => topics::_the_turn_we_lean_on(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "_answer_about" => topics::_answer_about(&a.s(0, "line"), &a.mem(1, "memory"), a.int(2, "keep", 6)).py(),
            "answer_about" => topics::answer_about(&a.s(0, "line"), &a.mem(1, "memory")).py(),
            "_the_trip_of_theirs" => topics::_the_trip_of_theirs(&a.s(0, "text")).into_iter().collect::<HashSet<_>>().py(),
            "the_place_they_are_on" => topics::the_place_they_are_on(&a.mem(0, "memory")).into_iter().collect::<HashSet<_>>().py(),
            "about_a_place_they_left_behind" => topics::about_a_place_they_left_behind(&a.s(0, "answer"), &a.mem(1, "memory")).py(),
            "same_as_before" => topics::same_as_before(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "pairing" => topics::pairing(&a.s(0, "text")).py(),
            "_doing_word" => topics::_doing_word(&a.s(0, "text")).py(),
            "_opening_instruction" => topics::_opening_instruction(&a.s(0, "sentence"), a.b(1, "allow_pointer", false)).py(),
            "_doing_part" => topics::_doing_part(&a.strs(0, "sentences"), &a.mem(1, "memory")).py(),
            "_says_it_again" => topics::_says_it_again(&a.s(0, "part"), &a.s(1, "sentence")).py(),
            "_the_rates_in" => topics::_the_rates_in(&a.s(0, "text")).into_iter().collect::<HashSet<_>>().py(),
            "_a_rate_that_fits" => topics::_a_rate_that_fits(&a.s(0, "line"), &a.s(1, "sentence")).py(),
            "_pick_amount" => topics::_pick_amount(&a.strs(0, "sentences"), &a.mem(1, "memory"), &a.s(2, "line")).py(),
            "_pick_sentence" => topics::_pick_sentence(&a.s(0, "line"), &a.und(1, "understood"), &a.s(2, "answer"), &a.mem(3, "memory"), a.b(4, "strict", false)).py(),
            "said_on_its_own" => {
                let s = a.j(0, "sentence");
                topics::said_on_its_own(s.as_str()).py()
            }
            "_the_sentence_it_answers" => topics::_the_sentence_it_answers(&a.s(0, "line"), &a.und(1, "understood"), &a.s(2, "answer"), &a.mem(3, "memory"), a.b(4, "strict", false)).py(),
            "_time_of" => topics::_time_of(&a.s(0, "sentence")).py(),
            "_option_range" => topics::_option_range(&a.s(0, "sentence")).py(),
            "_quoted" => topics::_quoted(&a.s(0, "part")).py(),
            "_the_option_for_the_topic" => topics::_the_option_for_the_topic(&a.s(0, "sentence"), &a.s(1, "line"), &a.mem(2, "memory")).py(),
            "_only_a_marker" => topics::_only_a_marker(&a.s(0, "part")).py(),
            "_duration_clauses" => topics::_duration_clauses(&a.s(0, "sentence")).py(),
            "_only_a_cadence" => topics::_only_a_cadence(&a.s(0, "part")).py(),
            "_a_window_in_the_weather" => topics::_a_window_in_the_weather(&a.s(0, "part")).py(),
            "_not_a_length" => topics::_not_a_length(&a.s(0, "part")).py(),
            "_duration_clause" => topics::_duration_clause(&a.s(0, "sentence")).py(),
            "_shorten" => topics::_shorten(&a.s(0, "part")).py(),
            "_asked_period" => topics::_asked_period(&a.s(0, "line")).py(),
            "_in_period" => topics::_in_period(&a.s(0, "part"), a.j(1, "period").as_i64()).py(),
            "_pick_duration" => topics::_pick_duration(&a.strs(0, "sentences"), &a.strs(1, "wanted"), &a.mem(2, "memory"), &a.s(3, "line")).py(),
            "_step_they_asked_for" => topics::_step_they_asked_for(&a.strs(0, "parts"), &a.strs(1, "wanted")).py(),
            "_time_in_words" => topics::_time_in_words(a.j(0, "minutes").as_f64().unwrap_or(0.0)).py(),
            "time_for_amount" => topics::time_for_amount(&a.s(0, "answer"), &a.s(1, "line")).py(),
            "_already_for_this_size" => topics::_already_for_this_size(&a.s(0, "line"), &a.s(1, "answer")).py(),
            "_in_grams" => topics::_in_grams(&a.s(0, "text")).py(),
            "_names_another_thing" => topics::_names_another_thing(&a.s(0, "line"), &a.s(1, "answer"), &a.strs(2, "about")).py(),
            "sized_reply" => topics::sized_reply(&a.s(0, "answer"), &a.s(1, "line"), &a.mem(2, "memory"), &a.strs(3, "about")).py(),
            "_sized_reply" => topics::_sized_reply(&a.s(0, "answer"), &a.s(1, "line"), &a.mem(2, "memory")).py(),
            "_times_asked" => topics::_times_asked(&a.s(0, "line"), &a.mem(1, "memory")).py(),
            "_rate_said" => topics::_rate_said(&a.s(0, "answer")).py(),
            "_phrase" => topics::_phrase(&a.s(0, "text")).py(),
            "has_time" => topics::has_time(&a.s(0, "text")).py(),
            "_worked_out_for_this_line" => topics::_worked_out_for_this_line(&a.und(0, "understood"), &a.s(1, "line"), &a.s(2, "answer")).py(),
            "asks_only" => topics::asks_only(&a.und(0, "understood")).py(),
            "_repeats_the_question" => topics::_repeats_the_question(&a.s(0, "line"), &a.s(1, "asked")).py(),
            "already_said" => {
                let t = a.j(1, "text");
                topics::already_said(&a.mem(0, "memory"), t.as_str(), a.int(2, "turns_back", 3)).py()
            }
            "_names_something_new" => topics::_names_something_new(&a.s(0, "line"), &a.s(1, "answer")).py(),
            "_named_a_fresh_thing" => {
                let rec = a.val(1, "record");
                topics::_named_a_fresh_thing(&a.s(0, "text"), (!rec.is_none()).then_some(&rec), &a.s(2, "answer")).py()
            }
            "left_for_a_blank" => topics::left_for_a_blank(&a.mem(0, "memory"), &a.s(1, "line")).py(),
            "_a_question" => topics::_a_question(&a.s(0, "text")).py(),
            "bare_question" => topics::bare_question(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "_still_the_topic" => topics::_still_the_topic(&a.val(0, "record"), &a.mem(1, "memory")).py(),
            "carried_topic" => topics::carried_topic(&a.mem(0, "memory"), &a.s(1, "text")).cloned().py(),
            "leans_on_topic" => topics::leans_on_topic(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "_only_the_topics_own_words" => topics::_only_the_topics_own_words(&a.s(0, "text"), &a.val(1, "record")).py(),
            "next_slot_reading" => topics::next_slot_reading(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "_introduction_covered" => topics::_introduction_covered(&a.s(0, "new"), &a.s(1, "old")).py(),
            "introduction_question" => topics::introduction_question(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "the_weather_ask" => topics::the_weather_ask(&a.und(0, "understood")).py(),
            "the_place_and_month" => topics::the_place_and_month(&a.s(0, "text")).py(),
            "_the_period_in" => topics::_the_period_in(&a.s(0, "text")).py(),
            "_a_period_and_nothing_else" => topics::_a_period_and_nothing_else(&a.s(0, "line")).py(),
            "the_period_they_ask_about" => topics::the_period_they_ask_about(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "_the_answer_was_about_the_weather" => topics::_the_answer_was_about_the_weather(&a.mem(0, "memory")).py(),
            "_the_place_the_chat_is_on" => topics::_the_place_the_chat_is_on(&a.s(0, "line"), &a.mem(1, "memory")).py(),
            "_with_the_period" => topics::_with_the_period(&a.s(0, "asked"), &a.s(1, "was"), &a.s(2, "now")).py(),
            "_the_meal_in" => topics::_the_meal_in(&a.s(0, "text")).py(),
            "_a_meal_and_nothing_else" => topics::_a_meal_and_nothing_else(&a.s(0, "line")).py(),
            "the_meal_they_ask_about" => topics::the_meal_they_ask_about(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "meal_follow_up" => topics::meal_follow_up(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "the_weather_of_the_trip" => topics::the_weather_of_the_trip(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "_the_trip_said" => topics::_the_trip_said(&a.s(0, "line"), &a.mem(1, "memory")).py(),
            "_the_lines_said" => topics::_the_lines_said(&a.mem(0, "memory")).py(),
            "a_placeholder_pronoun" => topics::a_placeholder_pronoun(&a.s(0, "line")).py(),
            "the_ask_about_the_thing_it_points_at" => topics::the_ask_about_the_thing_it_points_at(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "the_pronoun_written_as_what_it_stands_for" => topics::the_pronoun_written_as_what_it_stands_for(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "_the_thing_with_a_determiner" => topics::_the_thing_with_a_determiner(&a.s(0, "thing")).py(),
            "the_thing_it_is" => topics::the_thing_it_is(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "the_prognosis_the_line_asks" => topics::the_prognosis_the_line_asks(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "_keyed_on_how_long" => topics::_keyed_on_how_long(a.entry(0, "entry")?).py(),
            "_the_thing_as_a_question_names_it" => topics::_the_thing_as_a_question_names_it(&a.strs(0, "named")).py(),
            "the_property_of_the_thing" => topics::the_property_of_the_thing(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "the_limit_of_the_thing" => topics::the_limit_of_the_thing(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "_the_limit_written_out" => topics::_the_limit_written_out(&a.s(0, "line"), &a.s(1, "thing"), &a.mem(2, "memory")).py(),
            "_a_doing_word" => topics::_a_doing_word(&a.s(0, "word")).py(),
            "_fills" => topics::_fills(&a.s(0, "line"), &a.s(1, "thing")).py(),
            "the_airport_there_is" => topics::the_airport_there_is(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "the_animal_the_subject_is" => topics::the_animal_the_subject_is(a.und_opt(0, "understood").as_ref(), &a.mem(1, "memory")).py(),
            "_the_kind_of_animal" => topics::_the_kind_of_animal(&a.s(0, "text")).py(),
            "answers_about_another_creature" => topics::answers_about_another_creature(&a.s(0, "answer"), &a.s(1, "what_the_line_is")).py(),
            "_called" => topics::_called(&a.s(0, "who")).py(),
            "_the_pet_the_turns_before_named" => topics::_the_pet_the_turns_before_named(&a.mem(0, "memory"), a.int(1, "back", 3)).py(),
            "the_pet_the_line_before_named" => topics::the_pet_the_line_before_named(&a.mem(0, "memory")).py(),
            "a_detail_of_the_thing_we_hold" => topics::a_detail_of_the_thing_we_hold(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "_the_kind_the_line_leaves_out" => topics::_the_kind_the_line_leaves_out(&a.und(0, "understood"), &a.mem(1, "memory")).map(|(x, y)| (x, y)).py(),
            "the_thing_a_detail_belongs_to" => topics::the_thing_a_detail_belongs_to(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "_carrying_on" => topics::_carrying_on(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "_what_the_turn_took_away" => topics::_what_the_turn_took_away(&a.val(0, "record"), &a.mem(1, "memory")).py(),
            "_widened" => topics::_widened(&a.s(0, "line"), &a.und(1, "understood"), &a.val(2, "record"), &a.mem(3, "memory")).py(),
            "_the_length_of_the_aim" => topics::_the_length_of_the_aim(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "_a_gathering_read_as_the_food" => topics::_a_gathering_read_as_the_food(&a.und(0, "understood"), &a.s(1, "line"), &a.mem(2, "memory")).py(),
            "_the_whole_topic" => topics::_the_whole_topic(&a.s(0, "line"), &a.s(1, "record_text"), &a.mem(2, "memory")).py(),
            "_the_answer_is_about_what_the_line_names" => topics::_the_answer_is_about_what_the_line_names(a.entry(0, "entry")?, &a.s(1, "line")).py(),
            "_already_answered_here" => topics::_already_answered_here(a.entry(0, "entry")?, &a.mem(1, "memory")).py(),
            "_how_much_of_the_thing_the_entry_uses" => topics::_how_much_of_the_thing_the_entry_uses(a.entry(0, "entry")?, &a.s(1, "said")).py(),
            "_the_things_the_chat_named" => topics::_the_things_the_chat_named(&a.s(0, "record_text"), &a.mem(1, "memory"), &a.s(2, "line")).py(),
            "followup_reading" => topics::followup_reading(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "the_advice_ask_with_its_doing" => topics::the_advice_ask_with_its_doing(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            "_the_reply_to_the_turn_before" => topics::_the_reply_to_the_turn_before(&a.mem(0, "memory")).py(),
            "_the_step_back_is_undone" | "the_step_back_takes_its_words_back" | "_the_detail_steps_back" => {
                let mut m = a.mem(0, "memory");
                let r = match f {
                    "_the_step_back_is_undone" => topics::_the_step_back_is_undone(&mut m).py(),
                    "the_step_back_takes_its_words_back" => topics::the_step_back_takes_its_words_back(&mut m, &a.s(1, "reply")).py(),
                    _ => topics::_the_detail_steps_back(&mut m, &a.s(1, "reply")).py(),
                };
                return Some((r, Some(J::Object(m.to_plain()))));
            }
            "_a_step_back" => topics::_a_step_back(&a.s(0, "reply")).py(),
            _ => return None,
        },
        ("bot.memory.thread", f) => match f {
            "earlier_turn" => thread::earlier_turn(&a.mem(0, "memory")).py(),
            "only_the_arrival" => thread::only_the_arrival(&a.s(0, "text")).py(),
            "signs_off" => thread::signs_off(&a.s(0, "text")).py(),
            "carries_subject" => thread::carries_subject(&a.s(0, "text")).py(),
            "thin" => thread::thin(&a.s(0, "text")).py(),
            "points_back" => thread::points_back(&a.s(0, "text")).py(),
            "points_at_a_word_alone" => thread::points_at_a_word_alone(&a.s(0, "text")).py(),
            "asks_what_i_said" => thread::asks_what_i_said(&a.s(0, "text")).py(),
            "nothing_said" => thread::nothing_said(&a.mem(0, "memory")).py(),
            "_is_subject_phrase" => thread::_is_subject_phrase(&a.s(0, "phrase")).py(),
            "_from_topics" => thread::_from_topics(&a.mem(0, "memory")).py(),
            "_is_wh" => thread::_is_wh(&a.s(0, "word")).py(),
            "_strip_lead" => thread::_strip_lead(&a.strs(0, "words")).py(),
            "_with_you" => thread::_with_you(&a.strs(0, "words")).py(),
            "_first_person_aux" => thread::_first_person_aux(&a.s(0, "word")).py(),
            "_flip" => thread::_flip(&a.strs(0, "words"), a.b(1, "addressee", false)).py(),
            "your_words" => thread::your_words(&a.s(0, "text")).py(),
            "_expand" => thread::_expand(&a.s(0, "text")).py(),
            "_np_end" => thread::_np_end(&a.strs(0, "words"), a.int(1, "start", 0), a.int(2, "most", 3)).py(),
            "_embed" => thread::_embed(&a.strs(0, "words"), &a.strs(1, "names")).py(),
            "_keep_possessives" => thread::_keep_possessives(&a.s(0, "clause"), &a.s(1, "clean")).py(),
            "_truncated" => thread::_truncated(&a.s(0, "clause"), &a.s(1, "clean")).py(),
            "_numbers" => thread::_numbers(&a.s(0, "text"), &a.s(1, "clause")).py(),
            "restate" => thread::_restate(&a.s(0, "text")).py(),
            "_past" => thread::_past(&a.s(0, "word")).py(),
            "_agree" => thread::_agree(&a.s(0, "verb"), &a.s(1, "aux")).py(),
            "_third" => thread::_third(&a.s(0, "verb")).py(),
            "_conditional" => thread::_conditional(&a.s(0, "clause")).py(),
            "_restore_case" => thread::_restore_case(&a.s(0, "text"), &a.s(1, "clause")).py(),
            "_capital" => thread::_capital(&a.s(0, "text")).py(),
            "on_the_table" => thread::on_the_table(&a.mem(0, "memory"), a.int(1, "limit", 3)).py(),
            "_said_as_a_state" => thread::_said_as_a_state(&a.s(0, "head"), &a.s(1, "text")).py(),
            "in_a_list" => thread::in_a_list(&a.val(0, "phrases")).py(),
            "about" | "topic_index" | "last_ask" | "last_said" => {
                let skip = a.j(1, "skip");
                if !skip.is_null() {
                    return None; // the store's own reader, replayed with the store's module
                }
                let m = a.mem(0, "memory");
                match f {
                    "about" => thread::about_with(&m, None).py(),
                    "topic_index" => thread::topic_index_with(&m, None).py(),
                    "last_ask" => thread::last_ask_with(&m, None, a.j(2, "not_before").as_i64()).py(),
                    _ => thread::last_said_with(&m, None).py(),
                }
            }
            "_only_a_detail" => thread::_only_a_detail(&a.s(0, "text")).py(),
            "named" => thread::named(&a.mem(0, "memory"), a.int(1, "limit", 2)).py(),
            "checking_the_verdict" => thread::checking_the_verdict(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "checking_a_fact" => thread::checking_a_fact(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_fact_from" => thread::_fact_from(&a.s(0, "answer")).py(),
            "_first_clause" => thread::_first_clause(&a.s(0, "said"), a.int(1, "limit", 13)).py(),
            "_a_done_word" => thread::_a_done_word(&a.s(0, "word")).py(),
            "_the_verdict_they_gave" => thread::_the_verdict_they_gave(&a.s(0, "line")).py(),
            "_the_thing_they_had" => thread::_the_thing_they_had(&a.s(0, "line")).py(),
            "went" => thread::went(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_asked_already" => thread::_asked_already(&a.s(0, "reply")).py(),
            _ => return None,
        },
        ("bot.memory.named", f) => match f {
            "_answers_the_bot" => named::_answers_the_bot(&a.s(0, "text")).py(),
            "_previous_ask" => named::_previous_ask(&a.mem(0, "memory")).py(),
            "takes_back" => named::takes_back(&a.s(0, "text")).py(),
            "the_one_asked_about" => named::the_one_asked_about(&a.s(0, "text")).py(),
            "thing" => named::thing(&a.s(0, "text")).py(),
            "_looks_like_a_thing" => named::_looks_like_a_thing(&a.s(0, "phrase")).py(),
            "carries_a_name" => named::carries_a_name(&a.s(0, "text")).py(),
            "the_name_in" => named::the_name_in(&a.val(0, "value")).py(),
            "asked_for_a_name" => named::asked_for_a_name(&a.mem(0, "memory")).py(),
            "the_name_given" => named::the_name_given(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_not_got" => named::_not_got(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_said_the_food" => named::_said_the_food(&a.mem(0, "memory"), &a.s(1, "food")).py(),
            "the_thing_named" => named::the_thing_named(&a.mem(0, "memory")).py(),
            "about_the_thing" => named::about_the_thing(&a.s(0, "text"), &a.mem(1, "memory"), &a.s(2, "owner")).py(),
            "_the_thing_they_named" => named::_the_thing_they_named(&a.mem(0, "memory")).py(),
            "a_measure_asked_again" => named::a_measure_asked_again(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_a_length_ask" => named::_a_length_ask(&a.s(0, "text")).py(),
            "_a_length_ask_of_the_aim" => named::_a_length_ask_of_the_aim(&a.s(0, "text")).py(),
            "_answers_a_pace" => named::_answers_a_pace(&a.s(0, "ask"), &a.mem(1, "memory")).py(),
            "_the_entry" => named::_the_entry(&a.s(0, "ask"), &a.mem(1, "memory")).py(),
            "_answers_a_length" => named::_answers_a_length(&a.s(0, "ask"), &a.mem(1, "memory")).py(),
            "_the_aim_and_its_length_question" => named::_the_aim_and_its_length_question(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "a_goal_measured" => named::a_goal_measured(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "a_length_about_an_aim_on_a_deadline" => named::a_length_about_an_aim_on_a_deadline(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_no_thanks" => named::_no_thanks(&a.s(0, "text")).py(),
            "follow_up" => named::follow_up(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "names_nothing_to_be_about" => named::names_nothing_to_be_about(&a.s(0, "said")).py(),
            "_only_how_they_feel" => named::_only_how_they_feel(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "about_it" => named::about_it(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "a_figure_on_the_doing" => named::a_figure_on_the_doing(&a.mem(0, "memory"), &a.s(1, "word")).py(),
            "is_a_progress_ask" => named::is_a_progress_ask(&a.s(0, "text")).py(),
            "_the_doing" => named::_the_doing(&a.s(0, "word"), &a.mem(1, "memory")).py(),
            "the_doing_the_plan_named" => named::the_doing_the_plan_named(&a.s(0, "word"), &a.mem(1, "memory")).py(),
            "_the_plan_said" => named::_the_plan_said(&a.mem(0, "memory"), &a.s(1, "word")).py(),
            "_the_ing" => named::_the_ing(&a.s(0, "word")).py(),
            "_the_doing_in_hand" => named::_the_doing_in_hand(&a.mem(0, "memory")).py(),
            "the_duration_for_the_doing" => named::the_duration_for_the_doing(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_the_aspect_asked" => named::_the_aspect_asked(&a.strs(0, "low"), &a.s(1, "thing")).py(),
            "_the_job_before" => named::_the_job_before(&a.mem(0, "memory")).py(),
            "the_best_time_to_do_it" => named::the_best_time_to_do_it(&a.s(0, "text"), &a.und(1, "understood"), &a.mem(2, "memory")).py(),
            "_only_the_measure" => named::_only_the_measure(&a.strs(0, "low")).py(),
            "about_the_subject" => named::about_the_subject(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_without_and_at_the_start" => named::_without_and_at_the_start(&a.s(0, "said")).py(),
            "_the_doing_word" => named::_the_doing_word(&a.s(0, "text")).py(),
            "_what_the_chat_is_on" => named::_what_the_chat_is_on(&a.mem(0, "memory"), &a.s(1, "this_line")).py(),
            "_the_thing_in_hand" => named::_the_thing_in_hand(&a.mem(0, "memory"), &a.s(1, "this_line")).py(),
            "_the_place_phrase" => named::_the_place_phrase(&a.strs(0, "low")).py(),
            "_thing_in_hand" => named::_thing_in_hand(&a.mem(0, "memory")).py(),
            "_only_a_particle" => named::_only_a_particle(&a.s(0, "word"), &a.s(1, "text")).py(),
            "_the_answers_own_subject" => named::_the_answers_own_subject(&a.val(0, "answer")).py(),
            "reported_states" => named::reported_states(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_the_persons_own_words" => named::_the_persons_own_words(&a.mem(0, "memory")).py(),
            "stated_about_the_person" => named::stated_about_the_person(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "_a_place_of_its_own" => named::_a_place_of_its_own(&a.mem(0, "memory")).py(),
            "subject_words" => named::subject_words(&a.mem(0, "memory"), &a.s(1, "text")).py(),
            "care_reports" => named::care_reports(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "a_faulty_thing" => named::a_faulty_thing(&a.mem(0, "memory")).py(),
            "_the_answer_gave_a_time" => named::_the_answer_gave_a_time(&a.mem(0, "memory")).py(),
            "the_thing" => named::the_thing(&a.mem(0, "memory")).py(),
            "subject_word" => named::subject_word(&a.mem(0, "memory")).py(),
            "_the_noun_in" => named::_the_noun_in(&a.s(0, "said")).py(),
            "_the_record_before" => named::_the_record_before(&a.mem(0, "memory")).py(),
            "a_newer_thing" => named::a_newer_thing(&a.mem(0, "memory"), &a.s(1, "word")).py(),
            "_the_trip_place" => named::_the_trip_place(&a.mem(0, "memory")).py(),
            "_the_opening_place" => named::_the_opening_place(&a.mem(0, "memory")).py(),
            "_the_opening_place_at" => named::_the_opening_place_at(&a.mem(0, "memory")).py(),
            "_the_place_the_bot_just_named" => named::_the_place_the_bot_just_named(&a.mem(0, "memory")).py(),
            "a_part_of_the_trip" => named::a_part_of_the_trip(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "asks_a_part_of_the_trip" => named::asks_a_part_of_the_trip(&a.s(0, "text"), &a.mem(1, "memory")).py(),
            "the_food_it_is_about" => named::the_food_it_is_about(&a.und(0, "understood"), &a.mem(1, "memory")).py(),
            _ => return None,
        },
        _ => return None,
    };
    Some((r, None))
}

/// The recorded result in the form the Rust result is compared in.
fn expected(fname: &str, result: &J) -> J {
    match fname {
        // a reading: the keys a reader writes
        "followup_reading" | "next_slot_reading" | "meal_follow_up" | "the_ask_about_the_thing_it_points_at"
        | "the_pronoun_written_as_what_it_stands_for" | "the_thing_it_is" | "the_prognosis_the_line_asks"
        | "the_property_of_the_thing" | "the_limit_of_the_thing" | "the_airport_there_is" | "the_animal_the_subject_is"
        | "the_thing_a_detail_belongs_to" | "_carrying_on" | "_the_length_of_the_aim" | "the_advice_ask_with_its_doing" => {
            reading_view(result)
        }
        _ => result.clone(),
    }
}

fn main() {
    rsbot::runtime::quiet_panics();
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let path = &argv[0];
    let show: usize = argv.iter().position(|a| a == "--show").map(|i| argv[i + 1].parse().unwrap()).unwrap_or(3);
    let only: Option<&String> = argv.iter().position(|a| a == "--only").map(|i| &argv[i + 1]);
    let reader = std::io::BufReader::new(GzDecoder::new(std::fs::File::open(path).unwrap()));
    let mut stats: BTreeMap<String, (usize, usize, usize, usize)> = BTreeMap::new(); // ok, bad, panic, skipped
    let mut shown: BTreeMap<String, usize> = BTreeMap::new();
    for line in reader.lines() {
        let rec: J = serde_json::from_str(&line.unwrap()).unwrap();
        let name = rec["fn"].as_str().unwrap().to_string();
        if only.is_some_and(|o| !name.ends_with(o.as_str())) {
            continue;
        }
        let a = Args {
            args: rec["args"].as_array().cloned().unwrap_or_default(),
            kwargs: rec["kwargs"].as_object().cloned().unwrap_or_default(),
            alias: rec["alias"].as_u64().map(|x| x as usize),
        };
        let fname = name.split(':').nth(1).unwrap().to_string();
        let entry = stats.entry(name.clone()).or_default();
        let got = rsbot::runtime::guarded(|| call(&name, &a));
        let (ok, detail) = match got {
            None => {
                entry.2 += 1;
                (false, format!("panic: {}", rsbot::runtime::take_last_panic().unwrap_or_default()))
            }
            Some(None) => {
                entry.3 += 1;
                continue;
            }
            Some(Some((result, mem_after))) => {
                let raised = !rec["raised"].is_null();
                let want = expected(&fname, &rec["result"]);
                let got = expected(&fname, &result);
                let mut ok = !raised && norm(&want) == norm(&got);
                let mut detail = format!("want {}\n      got  {}", want, got);
                if raised {
                    detail = format!("python raised {}; got {}", rec["raised"], got);
                }
                if let (Some(after), Some(want_after)) = (mem_after, rec.get("mem_after")) {
                    let (wa, ga) = (memory_view(want_after), memory_view(&after));
                    if wa != ga {
                        ok = false;
                        detail = format!("{detail}\n      mem want {wa}\n      mem got  {ga}");
                    }
                }
                if ok {
                    entry.0 += 1;
                } else {
                    entry.1 += 1;
                }
                (ok, detail)
            }
        };
        if !ok {
            let n = shown.entry(name.clone()).or_default();
            if *n < show {
                *n += 1;
                let args_s = serde_json::to_string(&rec["args"]).unwrap();
                let args_s: String = args_s.chars().take(400).collect();
                println!("MISMATCH {name}\n  args {args_s}\n      {detail}");
            }
        }
    }
    let (mut tok, mut tbad, mut tpanic, mut tskip) = (0, 0, 0, 0);
    for (name, (ok, bad, panic, skip)) in &stats {
        tok += ok;
        tbad += bad;
        tpanic += panic;
        tskip += skip;
        if *bad > 0 || *panic > 0 {
            println!("{name}: ok {ok} bad {bad} panic {panic} skipped {skip}");
        }
    }
    println!("TOTAL ok {tok} bad {tbad} panic {tpanic} skipped {tskip} over {} functions", stats.len());
}
