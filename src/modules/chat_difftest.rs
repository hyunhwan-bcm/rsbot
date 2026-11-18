//! Differential check of MOD2's pure readers against PyBot's own outputs on real lines.
//!
//! The cases come from a Python script run over the turn oracles' lines (one record per line,
//! every reader's output by name). Run with `MOD2_DIFF=<cases.jsonl> cargo test --profile fast
//! --lib modules::chat::diff_tests -- --nocapture`; without the variable the test does nothing.

use super::*;
use serde_json::{json, Value as J};
use std::collections::BTreeMap;

fn span(s: &str, sp: Option<(usize, usize)>) -> J {
    match sp {
        Some((a, b)) => json!([text::char_index(s, a), text::char_index(s, b)]),
        None => J::Null,
    }
}

fn opt<T: serde::Serialize>(v: Option<T>) -> J {
    match v {
        Some(v) => json!(v),
        None => J::Null,
    }
}

fn val(v: &Value) -> J {
    v.to_plain()
}

fn rust_outputs(t: &str, c: &str, k: &[String]) -> BTreeMap<&'static str, J> {
    let mut o: BTreeMap<&'static str, J> = BTreeMap::new();
    let last = k.last().cloned().unwrap_or_default();
    let set_sorted = |s: HashSet<String>| {
        let mut v: Vec<String> = s.into_iter().collect();
        v.sort();
        json!(v)
    };
    o.insert("_tokens", json!(_tokens(t)));
    o.insert("_only_a_refusal", json!(_only_a_refusal(c, k)));
    o.insert("_keeps_failing_theirs", span(c, _keeps_failing_theirs(c)));
    o.insert("_letting_someone_down", span(c, _letting_someone_down(c)));
    o.insert("_about_themselves", json!(_about_themselves(k)));
    o.insert("_themselves_unfair", json!(_themselves_unfair(k)));
    o.insert("_judged_as_a_noun", json!(_judged_as_a_noun(k)));
    o.insert("_judged_a_role", json!(_judged_a_role(c)));
    o.insert("_not_a_night_out", json!(_not_a_night_out(c)));
    o.insert("_poor_word", opt(_poor_word(c)));
    o.insert("_line_as_read", json!(_line_as_read(t)));
    o.insert("_their_own_call", opt(_their_own_call(t)));
    o.insert("_fond_of_the_bot", json!(_fond_of_the_bot(c)));
    o.insert("_praised_the_answer", json!(_praised_the_answer(c)));
    o.insert("_a_stretch_that_drags", json!(_a_stretch_that_drags(c)));
    o.insert("_doing_something_to_a_thing", json!(_doing_something_to_a_thing(c)));
    o.insert("_asks_about_plans", json!(_asks_about_plans(c)));
    o.insert("_a_feeling_about_me", json!(_a_feeling_about_me(&last)));
    o.insert("_nothing_much_span", opt(_nothing_much_span(k).map(|(a, b)| vec![a, b])));
    o.insert("_says_nothing_much", json!(_says_nothing_much(k, c)));
    o.insert("_ing", if k.is_empty() { J::Null } else { json!(_ing(&last)) });
    o.insert("_is_greeting", json!(_is_greeting(k)));
    o.insert("_denies_a_good_word", json!(_denies_a_good_word(k)));
    o.insert("_is_thanks", json!(_is_thanks(k)));
    o.insert("_without_the_at_all", json!(_without_the_at_all(k)));
    o.insert("_without_the_nod", json!(_without_the_nod(k)));
    o.insert("_is_bye", json!(_is_bye(k)));
    o.insert("_bye_kind", json!(_bye_kind(k)));
    o.insert("_worry_kind", json!(_worry_kind(c)));
    o.insert("_without_skills", json!(_without_skills(c)));
    o.insert("_bare_feeling", opt(_bare_feeling(c)));
    o.insert("_own_state", json!(_own_state(c)));
    o.insert("_owner", opt(_owner(&text::split_ref(c))));
    o.insert("_hard_about_their_thing", json!(_hard_about_their_thing(c)));
    o.insert("_my_own_tears", json!(_my_own_tears(c)));
    o.insert("_a_low_word", json!(_a_low_word(c)));
    o.insert("_not_coping", json!(_not_coping(c)));
    o.insert("_speaker_going_mad", json!(_speaker_going_mad(c)));
    o.insert("_bad_day", json!(_bad_day(c)));
    o.insert("_scattered", json!(_scattered(c)));
    o.insert("_a_bare_acknowledgement", json!(_a_bare_acknowledgement(c)));
    o.insert("_apart_from_people", json!(_apart_from_people(c)));
    o.insert("_offered_a_choice", json!(_offered_a_choice(t)));
    o.insert("_asked_how_it_went", json!(_asked_how_it_went(c)));
    o.insert("_a_judgement_asked", json!(_a_judgement_asked(c)));
    o.insert("_is_a_nod", json!(_is_a_nod(t)));
    o.insert("_going_on", json!(_going_on(c)));
    o.insert("_said_about_themselves", json!(_said_about_themselves(c)));
    o.insert("_against_me", opt(_against_me(c)));
    o.insert("_a_thing_cannot", json!(_a_thing_cannot(c)));
    o.insert("_close_nouns", set_sorted(_close_nouns(c)));
    o.insert("_a_row_person", json!(_a_row_person(c)));
    o.insert("_the_row_names_its_subject", json!(_the_row_names_its_subject(c)));
    o.insert("_a_teen_has_gone_quiet", json!(_a_teen_has_gone_quiet(c)));
    o.insert("_the_ask_in_the_line", json!(_the_ask_in_the_line(c)));
    o.insert("_row_pronoun", json!(_row_pronoun(c)));
    o.insert("_row_subject", json!(_row_subject(c)));
    o.insert("_still_about_me", json!(_still_about_me(c)));
    o.insert("_my_night", json!(_my_night(c)));
    o.insert("_the_thing", json!(_the_thing(c)));
    o.insert("_thing_that_stays", json!(_thing_that_stays(c)));
    o.insert("_fed_up_with", json!(_fed_up_with(c)));
    o.insert("_drop_tail", json!(_drop_tail(c)));
    o.insert("_tidy", json!(_tidy(c)));
    o.insert("_place_values", json!(_place_values(t).into_iter().map(|(k, v)| (k.to_string(), v)).collect::<BTreeMap<_, _>>()));
    o.insert("_the_visit_was_this_week", json!(_the_visit_was_this_week(c)));
    o.insert(
        "_a_thing_of_care",
        json!(_a_thing_of_care(c, k.len() % 2 == 0).into_iter().map(|(k, v)| (k.to_string(), v)).collect::<BTreeMap<_, _>>()),
    );
    o.insert("_ends_plural", json!(_ends_plural(c)));
    o.insert("_keep_the_letters", json!(_keep_the_letters(c)));
    o.insert("_names_a_person", json!(_names_a_person(k.first().map(String::as_str).unwrap_or(""))));
    o.insert("_head_word", json!(_head_word(c)));
    o.insert("_the_poor_describer", json!(_the_poor_describer(c)));
    o.insert("_the_thing_this_line_names", json!(_the_thing_this_line_names(c)));
    o.insert("_without_a_filler_tail", json!(_without_a_filler_tail(t)));
    o.insert("_verb_stem", if k.is_empty() { J::Null } else { json!(_verb_stem(&last)) });
    o.insert("_asks_outright", json!(_asks_outright(c)));
    o.insert("_a_constant_tiredness", json!(_a_constant_tiredness(c)));
    o.insert("_says_dont_know", json!(_says_dont_know(c, k)));
    // `len(c) // 2` in characters, as a byte offset into the same string.
    let half = text::byte_at(c, text::len(c) / 2);
    o.insert("_own_deed", json!(_own_deed(c, half)));
    o.insert("_reads_as_distress", json!(_reads_as_distress(t)));
    o.insert("_bare_ask", json!(_bare_ask(t)));
    o.insert("_words_only", json!(_words_only(t)));
    o.insert("_names_its_own_thing", json!(_names_its_own_thing(t, "my phone is broken and the car")));
    o.insert("_somebody_thinks_i_should", opt(_somebody_thinks_i_should(c).map(|(a, b)| vec![a, b])));
    o.insert("_somebody_recommended_a_thing", opt(_somebody_recommended_a_thing(c)));
    o.insert("_judgement_noun_when", json!(_when_not_why(k)));
    let u = Understood { kind: "question".into(), asks: vec!["how".into()], ..Understood::default() };
    o.insert("_without_the_feeling_avoided", json!(_without_the_feeling_avoided(c, Some(&u))));
    o.insert("_negated_tired", json!(_negated(c, "tired")));
    o.insert("_denies_tired", json!(_denies_an_earlier_word(c, "tired")));
    o.insert("_a_thing_in_front_flat", json!(_a_thing_in_front(c, "flat")));
    o.insert("_is_howareyou_shape", json!(HOW_ARE_YOU.search(&k.join(" ")).is_some()));
    o.insert("_leaves", json!(_leaves(k)));
    o.insert("_frequency_only", json!(FREQUENCY_ONLY.match_(c).is_some()));
    use super::super::{attribute_said_again as A, carers_turn as T, chasing_a_body as B, grief as G, live_times as L};
    o.insert("G._words", json!(G::_words(t)));
    o.insert("G._age", opt(G::_age(c)));
    o.insert("G._animal", opt(G::_animal(c)));
    o.insert("G._any_animal", opt(G::_any_animal(c)));
    o.insert("G._young", json!(G::_young(c, c)));
    let who = format!("your {}", if k.is_empty() { "dog".to_string() } else { last.clone() });
    o.insert("G._age_pool", json!(G::_age_pool(c, &who)));
    o.insert("G._asks", json!(G::_asks(c)));
    o.insert("A._their_attribute", json!(A::_their_attribute(c)));
    o.insert("A._stem", if k.is_empty() { J::Null } else { json!(A::_stem(&last)) });
    o.insert("A._a_time", json!(A::_a_time(t)));
    o.insert("A._a_place", json!(A::_a_place(t)));
    let subject: HashSet<String> = ["tin", "kitchen", "room"].iter().map(|s| s.to_string()).collect();
    o.insert("A._a_size", json!(A::_a_size(t, &subject)));
    o.insert("A._the_value", json!(A::_the_value(c)));
    let (slot, subj) = A::_the_slot_ask(c);
    o.insert("A._the_slot_ask", json!([slot, set_sorted(subj)]));
    o.insert("A._spoken", json!(A::_spoken(c)));
    o.insert("A._a_cost_line", json!(A::_a_cost_line(t)));
    o.insert("A._values", json!(A::_the_values_the_kind_takes("what size did i say", t)));
    o.insert("A._values_money", json!(A::_the_values_the_kind_takes("what price did i say", t)));
    o.insert("B._the_body", opt(B::_the_body(c, &B::_words(c)).map(|(a, b)| vec![a, b])));
    o.insert("B._names_its_own_person", json!(B::_names_its_own_person(c)));
    o.insert("L._service_named", opt(L::_service_named(c)));
    o.insert("T._reads_as_easing", json!(T::_reads_as_easing(c)));
    o.insert("T._reads_as_easing_but_worse", json!(T::_reads_as_easing_but_worse(c)));
    o.insert("T._an_abuse_thread", json!(T::_an_abuse_thread(t)));
    o.insert("T._the_answer_sent_them_somewhere", json!(T::_the_answer_sent_them_somewhere(t)));
    let _ = val;
    o
}

/// Python's `False`/`True`/`None` for a reader that returns a falsy value or a match.
fn same(py: &J, rs: &J) -> bool {
    if py == rs {
        return true;
    }
    match (py, rs) {
        (J::Number(a), J::Number(b)) => a.as_f64() == b.as_f64(),
        (J::Array(a), J::Array(b)) => a.len() == b.len() && a.iter().zip(b).all(|(x, y)| same(x, y)),
        (J::Object(a), J::Object(b)) => a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).is_some_and(|w| same(v, w))),
        _ => false,
    }
}

#[test]
fn diff_against_pybot() {
    let Ok(path) = std::env::var("MOD2_DIFF") else { return };
    crate::runtime::quiet_panics();
    let text_ = std::fs::read_to_string(&path).expect("cases file");
    let mut bad: BTreeMap<String, (usize, Vec<String>)> = BTreeMap::new();
    let mut total = 0usize;
    for line in text_.lines() {
        let rec: J = serde_json::from_str(line).unwrap();
        let t = rec["text"].as_str().unwrap();
        let c = rec["clean"].as_str().unwrap();
        let k: Vec<String> = rec["tokens"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
        let ours = match std::panic::catch_unwind(|| rust_outputs(t, c, &k)) {
            Ok(o) => o,
            Err(_) => {
                let msg = crate::runtime::take_last_panic().unwrap_or_default();
                bad.entry("<panic>".into()).or_default().0 += 1;
                bad.get_mut("<panic>").unwrap().1.push(format!("{t:?}: {msg}"));
                continue;
            }
        };
        for (name, py) in rec["out"].as_object().unwrap() {
            total += 1;
            let rs = ours.get(name.as_str()).cloned().unwrap_or(J::String("<missing>".into()));
            if py.get("raised").is_some() {
                continue;
            }
            if !same(py, &rs) {
                let e = bad.entry(name.clone()).or_default();
                e.0 += 1;
                if e.1.len() < 3 {
                    e.1.push(format!("{c:?}\n        py {py}\n        rs {rs}"));
                }
            }
        }
    }
    println!("{total} reader outputs compared");
    for (name, (n, examples)) in &bad {
        println!("  {name}: {n} differ");
        for e in examples.iter().take(3) {
            println!("      {e}");
        }
    }
    assert!(bad.is_empty(), "differences against PyBot");
}

fn rust_memory_outputs(t: &str, c: &str, k: &[String], m: &mut Memory, r: &str) -> BTreeMap<&'static str, J> {
    use super::super::{attribute_said_again as A, carers_turn as T, chasing_a_body as B, grief as G, live_times as L};
    let mut o: BTreeMap<&'static str, J> = BTreeMap::new();
    let _ = k;
    o.insert("_idk_kind", json!(_idk_kind(m)));
    let (met, plan) = _just_met(m);
    o.insert("_just_met", json!([met, plan]));
    o.insert("_after_a_plan", json!(_after_a_plan(m, c)));
    o.insert("_said_before", json!(_said_before(r, m)));
    o.insert("_the_verdict_in", opt(_the_verdict_in(c, m)));
    o.insert("_topic_still", json!(_a_topic_still_in_this_exchange(c, m)));
    o.insert("_nothing_was_heard", json!(_nothing_was_heard(m, c)));
    o.insert("_asks_what_they_said", json!(_asks_what_they_said(r, m)));
    o.insert("_a_hedge_answering_the_question", json!(_a_hedge_answering_the_question(m)));
    o.insert("_the_bot_just_asked_them", json!(_the_bot_just_asked_them(m)));
    o.insert("_a_plain_good_answer", json!(_a_plain_good_answer(c, m)));
    o.insert("_a_thank_you_taken_off", json!(_a_thank_you_taken_off(c, m)));
    o.insert("_the_newest_line_started_the_topic", json!(_the_newest_line_started_the_topic(m)));
    o.insert("_loss_recent", json!(_loss_recent(m)));
    o.insert("_low_mood_recent", json!(_low_mood_recent(m)));
    o.insert("said_loss", json!(said_loss(m)));
    o.insert("_in_a_work_thread_age", json!(AN_AGE_SAID.match_(text::strip(c)).is_some()));
    o.insert("_the_line_on_the_screen", json!(_the_line_on_the_screen(m)));
    o.insert("_last_was_joke", json!(_last_was_joke(m)));
    o.insert("_a_pick_is_in_hand", json!(_a_pick_is_in_hand(m)));
    o.insert("_the_small_child", opt(_the_small_child(m)));
    o.insert("_recent_plural", json!(_recent_plural(m)));
    o.insert("_the_same_cold_again", json!(_the_same_cold_again(c, m)));
    o.insert("_asked_about_the_effort", json!(_asked_about_the_effort(m)));
    o.insert("_the_answer_met_it_as_news", json!(_the_answer_met_it_as_news(m)));
    o.insert("_news_before", json!(_news_before(m)));
    o.insert("_the_part_of_day", opt(_the_part_of_day(m)));
    o.insert("_as_meant", json!(_as_meant(c, m.understood.as_ref())));
    o.insert("_as_said", json!(_as_said(c, m.understood.as_ref())));
    o.insert("_as_typed", json!(_as_typed(c, m)));
    o.insert("_the_words_they_meant", json!(_the_words_they_meant(c, m.understood.as_ref())));
    o.insert("_normalised_line", opt(_normalised_line(m)));
    o.insert("_repaired_line", opt(_repaired_line(m)));
    o.insert("_question_coming", json!(_question_coming(m.understood.as_ref())));
    o.insert("_the_not_knowing_has_a_subject", json!(_the_not_knowing_has_a_subject(m)));
    o.insert("_a_forecast", json!(_a_forecast(m)));
    o.insert("_about_bot_life", opt(_about_bot_life(m.understood.as_ref())));
    let u = m.understood.clone().unwrap_or_default();
    o.insert("_confusion_names_a_thing", json!(_confusion_names_a_thing(&u)));
    o.insert("_name", opt(_name(m)));
    o.insert("_the_same_answer_again_words", json!(_words_only(r)));
    o.insert("_frequency_of_the_last_trouble_shape", json!(FREQUENCY_ONLY.match_(c).is_some() && _loss_recent(m)));
    let row = m.get("row").cloned();
    o.insert("_names_the_person", json!(_names_the_person(c, row.as_ref())));
    o.insert("G._remembered", match G::_remembered(m) {
        Some(v) => val(&v),
        None => J::Null,
    });
    o.insert("G._still_here", json!(G::_still_here(m)));
    o.insert("A._the_slot_recall_last", json!(A::_the_last_reply(m)));
    o.insert("A._the_line_that_says_it", json!(A::_the_line_that_says_it(t, m)));
    o.insert("A._the_slot_recall", opt(A::_the_slot_recall(t, m)));
    o.insert("A._the_newest_cost_line", json!(m.turns.iter().any(|x| A::_a_cost_line(&x.user))));
    o.insert("B._the_body_before", opt(B::_the_body_before(m, c).map(|(a, b)| vec![a, b])));
    o.insert("B._the_person_before", json!(B::_the_person_before(m)));
    o.insert("L._their_own_time", json!(L::_their_own_time(c, m)));
    o.insert("L.respond", opt(L::respond(t, m)));
    o.insert("T._the_answer_above", match T::_the_answer_above(m) {
        Some(turn) => json!({"user": turn.user, "bot": turn.bot, "module": turn.module}),
        None => J::Null,
    });
    o.insert("T._the_thread", json!(T::_the_thread(m, &r.to_lowercase())));
    o.insert("T._the_words_for_the_one_who_did_it", val(&T::_the_words_for_the_one_who_did_it(m)));
    o
}

#[test]
fn diff_memory_readers_against_pybot() {
    let Ok(path) = std::env::var("MOD2_MEMDIFF") else { return };
    crate::runtime::quiet_panics();
    let text_ = std::fs::read_to_string(&path).expect("cases file");
    let mut bad: BTreeMap<String, (usize, Vec<String>)> = BTreeMap::new();
    let mut total = 0usize;
    for line in text_.lines() {
        let rec: J = serde_json::from_str(line).unwrap();
        let t = rec["text"].as_str().unwrap();
        let c = rec["clean"].as_str().unwrap();
        let r = rec["reply"].as_str().unwrap_or("");
        let k: Vec<String> = rec["tokens"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
        let mut m = Memory::from_plain(rec["mem"].as_object().unwrap());
        let ours = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| rust_memory_outputs(t, c, &k, &mut m, r))) {
            Ok(o) => o,
            Err(_) => {
                let msg = crate::runtime::take_last_panic().unwrap_or_default();
                let e = bad.entry("<panic>".into()).or_default();
                e.0 += 1;
                if e.1.len() < 5 {
                    e.1.push(format!("{t:?}: {msg}"));
                }
                continue;
            }
        };
        for (name, py) in rec["out"].as_object().unwrap() {
            let Some(rs) = ours.get(name.as_str()) else { continue };
            total += 1;
            if py.get("raised").is_some() {
                continue;
            }
            if !same(py, rs) {
                let e = bad.entry(name.clone()).or_default();
                e.0 += 1;
                if e.1.len() < 3 {
                    e.1.push(format!("{t:?}\n        py {py}\n        rs {rs}"));
                }
            }
        }
    }
    println!("{total} memory-reader outputs compared");
    for (name, (n, examples)) in &bad {
        println!("  {name}: {n} differ");
        for e in examples.iter().take(3) {
            println!("      {e}");
        }
    }
    assert!(bad.is_empty(), "differences against PyBot");
}
