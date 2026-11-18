//! One entry per checked function: the recorded arguments handed to the Rust port.
#![allow(unused_imports, dead_code)]
use super::{r, rm, rv, Args, Out, Res, Skip, ToJ};
use rsbot::data::Value;
use rsbot::memory as m;
use rsbot::py::text as pt;
use serde_json::{json, Value as J};

/// What Python's result looks like in the shape the checker's entry returns.
pub fn expected(row: &J, _name: &str) -> J {
    row["result"].clone()
}

/// Run the Rust port of one recorded call. `None` means the checker has no entry for it.
pub fn call(name: &str, a: &Args) -> Option<Res> {
    let (module, fname) = name.split_once(':')?;
    match module.strip_prefix("bot.memory.")? {
        "taste" => taste(fname, a),
        "healing" => healing(fname, a),
        "change" => change(fname, a),
        "times" => times(fname, a),
        "places" => places(fname, a),
        "people" => people(fname, a),
        "asleep" => asleep(fname, a),
        "back_again" => back_again(fname, a),
        "measure" => measure(fname, a),
        "outfit" => outfit(fname, a),
        "returns" => returns(fname, a),
        "aboutthem" => aboutthem(fname, a),
        "amounts" => amounts(fname, a),
        "another" => another(fname, a),
        "carried_on" => carried_on(fname, a),
        "company" => company(fname, a),
        "condition" => condition(fname, a),
        "earlier" => earlier(fname, a),
        "favourite" => favourite(fname, a),
        "guest" => guest(fname, a),
        "household" => household(fname, a),
        "matters" => matters(fname, a),
        "rates" => rates(fname, a),
        _ => None,
    }
}

/// A byte offset in `text` as Python's position (code points).
fn chars(text: &str, byte: usize) -> i64 {
    pt::char_index(text, byte) as i64
}

fn taste(fname: &str, a: &Args) -> Option<Res> {
    use m::taste as t;
    Some(match fname {
        "_kept" => r(t::_kept(a.s(0, "thing")).into_iter().map(|(i, w)| (i, w)).collect::<Vec<_>>()),
        "_clean" => r(t::_clean(a.s(0, "thing"))),
        "_read_clause" => {
            let text = a.s(0, "text");
            let (taste, end) = t::_read_clause(text);
            let squashed = pt::squash(text);
            r((taste, end.map(|e| chars(&squashed, e))))
        }
        "read" => r(t::read(a.s(0, "line"))),
        "read_all" => r(t::read_all(a.s(0, "line"))),
        "only_a_taste" => r(t::only_a_taste(a.s(0, "text"))),
        "said_about" => {
            let text = a.s(0, "text");
            a.pos_in(text, 1, "position").and_then(|p| r(t::said_about(text, p as i64)))
        }
        "asks_about_a_taste" => r(t::asks_about_a_taste(a.s(0, "line"))),
        "read_other" => r(t::read_other(a.s(0, "line"))),
        _ => return None,
    })
}

fn sorted(set: std::collections::HashSet<String>) -> Vec<String> {
    let mut v: Vec<String> = set.into_iter().collect();
    v.sort();
    v
}

fn healing(fname: &str, a: &Args) -> Option<Res> {
    use m::healing as h;
    Some(match fname {
        "asks" => r(h::asks(a.s(0, "text"))),
        "the_timeline" => r(h::the_timeline(a.s(0, "answer"))),
        "_the_same_length" => r(h::_the_same_length(a.s(0, "reply"), a.s(1, "timeline"))),
        "_stem" => r(h::_stem(a.s(0, "word"))),
        "_carries" => r(sorted(h::_carries(a.s(0, "text")))),
        "_said_before" => r(h::_said_before(&a.memory(0, "memory"), a.n_or(1, "keep", 2))),
        "derived" => r(h::derived(&a.memory(0, "memory"))),
        "_no_timeline" => r(h::_no_timeline(&a.memory(0, "memory"), a.s(1, "step"))),
        "_a_hedge" => r(h::_a_hedge(a.s(0, "reply"))),
        "_the_card" => r(h::_the_card(&a.memory(0, "memory"))),
        "vetted" => {
            let mut memory = a.memory(1, "memory");
            let step_of = match a.func(3, "step_of") {
                Some(id) => Value::Fn(std::sync::Arc::new(rsbot::data::FnRef { id: id.to_string(), cells: vec![], defaults: vec![] })),
                None => Value::None,
            };
            let reply = h::vetted(a.s(0, "text"), &mut memory, a.opt_s(2, "reply"), &step_of);
            rm(reply, 1, &memory)
        }
        "_the_same_as_the_card" => r(h::_the_same_as_the_card(a.s(0, "said"), a.s(1, "card"))),
        "_the_healing_line" => r(h::_the_healing_line(a.s(0, "said"), a.s(1, "timeline"), a.s(2, "card"), &a.memory(3, "memory"), a.s(4, "step"))),
        _ => return None,
    })
}

fn change(fname: &str, a: &Args) -> Option<Res> {
    use m::change as c;
    Some(match fname {
        "_names" => r(c::_names(a.s(0, "line"), a.s(1, "subject"))),
        "_left_over" => r(c::_left_over(a.s(0, "line"), a.s(1, "subject"))),
        "reported" => r(c::reported(a.s(0, "text"), a.s(1, "subject"))),
        "reply" => {
            let was = a.v(2, "was");
            let was = if was.truthy() { m::mem7::py_str(&was) } else { String::new() };
            r(c::reply(a.s(0, "kind"), a.s(1, "subject"), &was))
        }
        _ => return None,
    })
}

fn times(fname: &str, a: &Args) -> Option<Res> {
    use m::times as t;
    Some(match fname {
        "_a_routine" => r(t::_a_routine(a.s(0, "phrase"), a.b(1, "ask", false))),
        "_the_times" => r(t::_the_times(&a.memory(0, "memory"))),
        "_keep_a_time" => {
            let mut memory = a.memory(0, "memory");
            t::_keep_a_time(&mut memory, a.s(1, "what"), a.s(2, "at"), a.s(3, "said"));
            rm((), 0, &memory)
        }
        "keep" => {
            let mut memory = a.memory(1, "memory");
            t::keep(a.s(0, "text"), &mut memory);
            rm((), 1, &memory)
        }
        "answer" => r(t::answer(a.s(0, "text"), &a.memory(1, "memory"))),
        "is_a_time_ask" => r(t::is_a_time_ask(a.s(0, "text"), &a.memory(1, "memory"))),
        _ => return None,
    })
}

/// A recorded reading (`understood`) as the typed one, or None.
fn reading(a: &Args, i: usize, name: &str) -> Result<Option<rsbot::understood::Understood>, Skip> {
    match a.get(i, name) {
        None | Some(J::Null) => Ok(None),
        Some(j) => serde_json::from_value(j.clone()).map(Some).map_err(|_| Skip::Shape),
    }
}

fn places(fname: &str, a: &Args) -> Option<Res> {
    use m::places as p;
    Some(match fname {
        "_a_place" => r(p::_a_place(a.s(0, "value"))),
        "_the_place_alone" => r(p::_the_place_alone(a.s(0, "text"))),
        "_said_places" => r(p::_said_places(&a.memory(0, "memory"))),
        "_a_town_moved" => r(p::_a_town_moved(&a.memory(0, "memory"), a.s(1, "text"))),
        "_other_place" => r(p::_other_place(&a.memory(0, "memory"), a.s(1, "who"))),
        "_asked_person" => {
            let mut memory = a.memory(0, "memory");
            let out = p::_asked_person(&mut memory, a.s(1, "low"));
            rm(out, 0, &memory)
        }
        _ => return None,
    })
}

fn people(fname: &str, a: &Args) -> Option<Res> {
    use m::people as p;
    Some(match fname {
        "_the_age_of_the_group" => r(p::_the_age_of_the_group(&a.memory(0, "memory"))),
        "_with_an_age" => r(p::_with_an_age(&a.v(0, "record"))),
        "_age" => r(p::_age(&a.v(0, "record"))),
        "_the_group_record" => r(p::_the_group_record(&a.memory(0, "memory"), a.s(1, "line"))),
        "_the_ones_a_them_points_at" => r(p::_the_ones_a_them_points_at(&a.vals(0, "people"))),
        "the_people" => {
            let mut memory = a.memory(0, "memory");
            let out = p::the_people(&mut memory);
            rm(out, 0, &memory)
        }
        "a_person_with_an_age" | "a_child_the_chat_is_about" | "the_age_of_the_person_they_mean" => {
            let understood = match reading(a, 1, "understood") {
                Ok(u) => u,
                Err(e) => return Some(Err(e)),
            };
            let mut memory = a.memory(0, "memory");
            let out = match fname {
                "a_person_with_an_age" => p::a_person_with_an_age(&mut memory, understood.as_ref()).j(),
                "a_child_the_chat_is_about" => p::a_child_the_chat_is_about(&mut memory, understood.as_ref()).j(),
                _ => p::the_age_of_the_person_they_mean(&mut memory, understood.as_ref()).j(),
            };
            rm(out, 0, &memory)
        }
        _ => return None,
    })
}

fn asleep(fname: &str, a: &Args) -> Option<Res> {
    use m::asleep as s;
    Some(match fname {
        "_end" => match a.get(0, "marker").and_then(|j| j.get("__re__")) {
            Some(p) => {
                let marker = rsbot::py::re::Re::new(p[0].as_str().unwrap_or(""), p[1].as_u64().unwrap_or(0) as u32);
                let text = a.s(1, "text");
                r(s::_end(&marker, text, a.b(2, "after_bed", false), a.n_or(3, "limit", 26)))
            }
            None => Err(Skip::Shape),
        },
        "_ends" => r(s::_ends(a.s(0, "line"), a.b(1, "wake_only", false))),
        "_last_night" => r(s::_last_night(&a.memory(0, "memory"))),
        "_a_length" => match a.get(0, "hours").and_then(J::as_f64) {
            Some(h) => r(s::_a_length(h)),
            None => Err(Skip::Shape),
        },
        "_a_word" => a.n(0, "number").and_then(|n| r(s::_a_word(n))),
        "_seen" => {
            let mut memory = a.memory(0, "memory");
            let out = s::_seen(&mut memory, a.s(1, "reply"));
            rm(out, 0, &memory)
        }
        "answer_for" => {
            let mut memory = a.memory(1, "memory");
            let out = s::answer_for(a.s(0, "text"), &mut memory);
            rm(out, 1, &memory)
        }
        _ => return None,
    })
}

fn back_again(fname: &str, a: &Args) -> Option<Res> {
    use m::back_again as b;
    Some(match fname {
        "_the_earlier_line" => r(b::_the_earlier_line(&a.memory(0, "memory"), a.s(1, "thing"), a.s(2, "text"))),
        "_the_thing_went_away" => r(b::_the_thing_went_away(&a.memory(0, "memory"), a.s(1, "thing"), a.s(2, "text"))),
        "_the_words" => {
            let pairs: serde_json::Map<String, J> = b::_the_words(a.b(0, "plural", false)).into_iter().map(|(k, v)| (k, J::String(v))).collect();
            r(J::Object(pairs))
        }
        "_filled" => r(b::_filled(a.s(0, "line"), a.s(1, "thing"), a.b(2, "plural", false))),
        "_an_earlier_topic" => r(b::_an_earlier_topic(&a.memory(0, "memory"), a.s(1, "thing"))),
        "_from_this_line" => r(b::_from_this_line(&a.v(0, "record"), &a.memory(1, "memory"))),
        "_the_thing" => r(b::_the_thing(a.s(0, "text"), &a.memory(1, "memory")).map(|(t, p)| (if t.is_empty() { J::Null } else { J::String(t) }, p))),
        "reply_for" => r(b::reply_for(a.s(0, "text"), &a.memory(1, "memory"))),
        _ => return None,
    })
}

fn measure(fname: &str, a: &Args) -> Option<Res> {
    use m::measure as x;
    Some(match fname {
        "the_measure_asked" => r(x::the_measure_asked(a.s(0, "line"))),
        "_the_card_said_last" => r(x::_the_card_said_last(&a.memory(0, "memory"))),
        "_the_clause_that_carries" => r(x::_the_clause_that_carries(a.s(0, "sentence"), None)),
        "_the_sentence_with" => r(x::_the_sentence_with(a.s(0, "text"), a.s(1, "measure"))),
        "_the_thing_the_line_named" => r(x::_the_thing_the_line_named(&a.memory(0, "memory"))),
        "_with_the_thing" => r(x::_with_the_thing(a.s(0, "sentence"), a.s(1, "thing"))),
        "_as_a_headline" => r(x::_as_a_headline(a.s(0, "thing"), a.s(1, "sentence"))),
        "_as_a_sentence" => r(x::_as_a_sentence(a.s(0, "text"))),
        "reply_for" => r(x::reply_for(a.s(0, "text"), &a.memory(1, "memory"))),
        _ => return None,
    })
}

fn outfit(fname: &str, a: &Args) -> Option<Res> {
    use m::outfit as o;
    Some(match fname {
        "_recent" => r(o::_recent(&a.memory(0, "memory"))),
        "_an" => r(o::_an(a.s(0, "phrase"))),
        "the_occasion" => r(o::the_occasion(&a.memory(0, "memory"))),
        "_the_last_answer" => r(o::_the_last_answer(&a.memory(0, "memory"))),
        "_the_answer_already_gave_it" => r(o::_the_answer_already_gave_it(&a.memory(0, "memory"))),
        "the_sentence_answered" => r(o::the_sentence_answered(a.s(0, "text"), &a.memory(1, "memory"))),
        "_names_an_occasion" => r(o::_names_an_occasion(a.s(0, "text"))),
        "_names_its_own_outing" => r(o::_names_its_own_outing(a.s(0, "text"))),
        "written_out" => r(o::written_out(a.s(0, "text"), &a.memory(1, "memory"))),
        _ => return None,
    })
}

fn returns(fname: &str, a: &Args) -> Option<Res> {
    use m::returns as x;
    Some(match fname {
        "_the_chat_has" => r(x::_the_chat_has(&a.strs(0, "words"), &a.memory(1, "memory"))),
        "_the_parts" => r(x::_the_parts(a.s(0, "text"))),
        "_the_lines" => r(x::_the_lines(&a.strs(0, "named"), a.s(1, "ask"))),
        "_the_domain_of_the_chat" => r(x::_the_domain_of_the_chat(&a.strs(0, "named"), &a.memory(1, "memory"))),
        "a_return_to_the_topic" => r(x::a_return_to_the_topic(a.s(0, "text"), &a.memory(1, "memory"))),
        "the_thing_named" => r(x::the_thing_named(a.s(0, "text"))),
        "_the_sentence_with_the_amount" => r(x::_the_sentence_with_the_amount(a.s(0, "answer"))),
        "as_the_same_answer" => r(x::as_the_same_answer(a.s(0, "reply"), a.s(1, "text"), &a.v(2, "short"))),
        "a_bare_return" => r(x::a_bare_return(a.s(0, "text"), a.opt_memory(1, "memory").as_ref())),
        _ => return None,
    })
}

/// A memory argument, the call, and the memory after it (argument `i`).
macro_rules! with_mem {
    ($a:expr, $i:expr, $name:expr, |$m:ident| $body:expr) => {{
        let mut $m = $a.memory($i, $name);
        let out = $body;
        rm(out, $i, &$m)
    }};
}

fn aboutthem(fname: &str, a: &Args) -> Option<Res> {
    use m::aboutthem as x;
    Some(match fname {
        "_the_child" => r(match x::_the_child(a.s(0, "text"), &a.memory(1, "memory")) {
            Some((how, who)) => json!([how, who]),
            None => json!([null, ""]),
        }),
        "_people" => r(x::_people(&a.memory(0, "memory"))),
        "with_the_age" => r(x::with_the_age(a.s(0, "text"), &a.memory(1, "memory"))),
        "an_age_ahead" => r(x::an_age_ahead(a.s(0, "text"), &a.memory(1, "memory"))),
        "_the_one_asked_about" => r(match x::_the_one_asked_about(a.s(0, "said"), &a.memory(1, "memory")) {
            Some((age, words)) => json!([age, words]),
            None => json!([null, ""]),
        }),
        "_an_age_said" => a.n(0, "age").and_then(|n| r(x::_an_age_said(n))),
        "_the_subject" => r(x::_the_subject(a.s(0, "words"))),
        "_the_future" => r(x::_the_future(a.s(0, "words"))),
        _ => return None,
    })
}

fn amounts(fname: &str, a: &Args) -> Option<Res> {
    use m::amounts as x;
    Some(match fname {
        "_a_value" => r(x::_a_value(a.s(0, "word"))),
        "_a_number" => match a.get(0, "value").and_then(J::as_f64) {
            Some(v) => r(x::_a_number(v)),
            None => Err(Skip::Shape),
        },
        "_counts" => r(x::_counts(a.s(0, "text"))),
        "a_count_they_report" => r(x::a_count_they_report(a.s(0, "text"))),
        "_the_figure_named" => r(x::_the_figure_named(a.s(0, "answer"), a.s(1, "thing"), a.get(2, "mine").and_then(J::as_f64))),
        "_the_periods_disagree" => r(x::_the_periods_disagree(a.opt_s(0, "mine"), a.opt_s(1, "theirs"))),
        "against_the_last_answer" => r(x::against_the_last_answer(&a.v(0, "said"), a.s(1, "answer"))),
        "_the_figure_said" => r(x::_the_figure_said(&a.v(0, "against"))),
        "_a_mid_sentence" => r(x::_a_mid_sentence(a.s(0, "figure"))),
        "_the_aim_said" => r(x::_the_aim_said(&a.v(0, "against"))),
        "the_comparison" => r(x::the_comparison(&a.v(0, "against"))),
        _ => return None,
    })
}

fn sorted_set(set: std::collections::HashSet<String>) -> Vec<String> {
    let mut v: Vec<String> = set.into_iter().collect();
    v.sort();
    v
}

fn another(fname: &str, a: &Args) -> Option<Res> {
    use m::another as x;
    Some(match fname {
        "names_nothing" => r(x::names_nothing(a.s(0, "text"))),
        "ruled_out_words" => r(sorted_set(x::ruled_out_words(a.s(0, "text")))),
        "_names_one" => r(x::_names_one(a.s(0, "part"), &a.strs(1, "ruled_out").into_iter().collect())),
        "split_sentences" => r(x::split_sentences(a.s(0, "said"))),
        "_sentences" => r(x::_sentences(a.s(0, "said"))),
        "_said_before" => r(x::_said_before(&a.memory(0, "memory"), a.s(1, "said"), a.get(2, "before").and_then(J::as_i64))),
        "_cut" => r(x::_cut(a.s(0, "said"))),
        "_a_caveat_about_getting_help" => r(x::_a_caveat_about_getting_help(a.s(0, "part"))),
        "is_a_step" => r(x::is_a_step(a.s(0, "part"))),
        "_the_rest" => r(x::_the_rest(a.s(0, "said"), &a.memory(1, "memory"), &a.strs(2, "ruled_out").into_iter().collect(), a.get(3, "at").and_then(J::as_i64))),
        "other_option" => r(x::other_option(a.s(0, "text"), &a.memory(1, "memory"))),
        _ => return None,
    })
}

fn carried_on(fname: &str, a: &Args) -> Option<Res> {
    use m::carried_on as x;
    Some(match fname {
        "_a_whole_step" => r(x::_a_whole_step(a.s(0, "said"))),
        "_stem" => r(x::_stem(a.s(0, "word"))),
        "_words" => r(x::_words(a.s(0, "text"))),
        "_steps" => r(x::_steps(a.s(0, "answer"))),
        "_doing" => r(x::_doing(a.s(0, "word"))),
        "_the_action" => r(x::_the_action(a.s(0, "line"))),
        "reply_for" => r(x::reply_for(a.s(0, "text"), &a.memory(1, "memory"))),
        _ => return None,
    })
}

fn company(fname: &str, a: &Args) -> Option<Res> {
    use m::company as x;
    Some(match fname {
        "_bare" => r(x::_bare(a.s(0, "who"))),
        "_the_companion" => r(x::_the_companion(a.s(0, "text"))),
        "_as_you_would_say_it" => r(x::_as_you_would_say_it(a.s(0, "who"))),
        "_more_than_one" => r(x::_more_than_one(a.s(0, "who"))),
        "_the_correction" => r(x::_the_correction(a.s(0, "text"), &a.memory(1, "memory"))),
        "_a_trip_is_here" => r(x::_a_trip_is_here(&a.memory(0, "memory"))),
        "_on_their_own" => r(x::_on_their_own(&a.memory(0, "memory"))),
        "_the_phrase_a_frame_can_say" => r(x::_the_phrase_a_frame_can_say(a.s(0, "plan"))),
        "_the_trip_on_the_table" => r(x::_the_trip_on_the_table(&a.memory(0, "memory"))),
        "_new_to_the_chat" => r(x::_new_to_the_chat(a.s(0, "who"), &a.memory(1, "memory"))),
        "_the_one_person_asked_about" => r(x::_the_one_person_asked_about(a.s(0, "text"), &a.memory(1, "memory"))),
        "_the_companion_joins" => r(x::_the_companion_joins(a.s(0, "text"), &a.memory(1, "memory"))),
        "_the_plan_is_this_line" => r(x::_the_plan_is_this_line(a.s(0, "text"), &a.memory(1, "memory"))),
        "observe" => with_mem!(a, 1, "memory", |mm| x::observe(a.s(0, "text"), &mut mm)),
        "answer" => r(x::answer(a.s(0, "text"), &a.memory(1, "memory"))),
        _ => return None,
    })
}

fn condition(fname: &str, a: &Args) -> Option<Res> {
    use m::condition as x;
    Some(match fname {
        "_places" => r(sorted_set(x::_places(&a.strs(0, "words")))),
        "_words" => r(x::_words(a.s(0, "text"))),
        "_pairs" => r(sorted_set(x::_pairs(&a.strs(0, "words")))),
        "about_someone_else" => r(x::about_someone_else(a.s(0, "text"))),
        "_sentences" => r(x::_sentences(a.s(0, "said"))),
        "_one_move" => r(x::_one_move(a.s(0, "consequence"))),
        "conditions" => r(x::conditions(a.s(0, "said"))),
        "_number" => r(x::_number(a.s(0, "word"))),
        "_measures" => {
            let o: serde_json::Map<String, J> = x::_measures(a.s(0, "text")).into_iter().map(|(k, v)| (k, json!(v))).collect();
            r(J::Object(o))
        }
        "_tests" => r(x::_tests(a.s(0, "case")).into_iter().map(|(u, v, s)| json!([u, v, s])).collect::<Vec<J>>()),
        "_meets" => r(x::_meets(a.s(0, "case"), a.s(1, "text"))),
        "chosen" => r(x::chosen(a.s(0, "text"), a.s(1, "said"))),
        "refused" => r(x::refused(a.s(0, "text"), a.s(1, "said"))),
        "_carried" => r(x::_carried(&a.strs(0, "case_words"), &a.strs(1, "words"))),
        "_about_a_person" => r(x::_about_a_person(&a.strs(0, "case_words"))),
        "_candidates" => r(x::_candidates(a.s(0, "text"), a.s(1, "said"))),
        _ => return None,
    })
}

fn said_j(found: Option<(i64, String)>) -> J {
    match found {
        Some((n, line)) => json!([n, line]),
        None => J::Null,
    }
}

fn earlier(fname: &str, a: &Args) -> Option<Res> {
    use m::earlier as x;
    let mem = || a.memory(0, "memory");
    Some(match fname {
        "_a_word" => r(x::_a_word(a.s(0, "word"))),
        "said_lines" => r(x::said_lines(&mem()).into_iter().map(|(n, l)| json!([n, l])).collect::<Vec<J>>()),
        "kind_of" => r(x::kind_of(a.s(0, "text"))),
        "carries" => r(x::carries(a.s(0, "kind"), a.s(1, "line"))),
        "_about" => r(x::_about(a.s(0, "text"))),
        "_said_words" => r(x::_said_words(a.s(0, "text"))),
        "_mentions" => r(x::_mentions(a.s(0, "line"), &a.strs(1, "words"))),
        "kinds_for" => r(x::kinds_for(a.s(0, "kind"), a.s(1, "ask"))),
        "carried" => {
            let window = a.get(3, "window").and_then(J::as_u64).map(|w| w as usize);
            r(said_j(x::carried(&mem(), a.s(1, "kind"), &a.strs(2, "about"), window, &a.strs(4, "prefer"), a.n_or(5, "after", 0))))
        }
        "_pointing_back" => r(x::_pointing_back(&mem(), &a.strs(1, "about"))),
        "carried_alone" => r(x::carried_alone(&mem(), a.s(1, "kind"))),
        "hand_back" => r(x::hand_back(a.s(0, "line"), a.opt_memory(1, "memory").as_ref(), a.s(2, "text"))),
        "a_claim" => r(x::a_claim(a.s(0, "reply"))),
        "_values" => r(sorted_set(x::_values(a.s(0, "kind"), a.s(1, "text")))),
        "gives_the_named_value" => r(x::gives_the_named_value(a.s(0, "text"), &a.memory(1, "memory"), a.s(2, "reply"))),
        "another_line_was_asked_for" => r(x::another_line_was_asked_for(a.s(0, "text"), &a.memory(1, "memory"), a.s(2, "reply"))),
        "a_misquoted_line" => r(x::a_misquoted_line(a.s(0, "reply"), &a.memory(1, "memory"), a.s(2, "ask"))),
        "answer_for" => r(x::answer_for(a.s(0, "text"), &a.memory(1, "memory"))),
        "their_own_words" => r(x::their_own_words(a.s(0, "text"))),
        "from_the_line" => r(x::from_the_line(a.s(0, "text"), &a.memory(1, "memory"))),
        "put_right" => r(x::put_right(a.s(0, "reply"), &a.memory(1, "memory"), a.s(2, "ask"))),
        _ => return None,
    })
}

fn favourite(fname: &str, a: &Args) -> Option<Res> {
    use m::favourite as x;
    Some(match fname {
        "_plain" => r(x::_plain(a.s(0, "value"))),
        "_a_value" => r(x::_a_value(a.s(0, "value"))),
        "_a_thing" => r(x::_a_thing(a.s(0, "thing"))),
        "_clauses" => r(x::_clauses(a.s(0, "text"))),
        "read" => r(x::read(a.s(0, "line"))),
        "asks_something_too" => r(x::asks_something_too(a.s(0, "line"))),
        "asked" => r(x::asked(a.s(0, "line"))),
        "_singular" => r(x::_singular(a.s(0, "thing"))),
        "asks_all" => r(x::asks_all(a.s(0, "line"))),
        "prefers_now" => r(x::prefers_now(a.s(0, "line"))),
        "an_answer" => r(x::an_answer(a.s(0, "line"), a.s(1, "thing"))),
        "corrected" => r(x::corrected(a.s(0, "line"), a.s(1, "thing"))),
        "kept" => r(x::kept(&a.memory(0, "memory"), a.s(1, "thing"))),
        "the_favourites" => r(x::the_favourites(&a.memory(0, "memory"))),
        "store" => with_mem!(a, 0, "memory", |mm| x::store(&mut mm, &a.v(1, "found"))),
        "their_words" => r(x::their_words(a.s(0, "value"))),
        "watch" => with_mem!(a, 1, "memory", |mm| x::watch(a.s(0, "text"), &mut mm)),
        _ => return None,
    })
}

fn guest(fname: &str, a: &Args) -> Option<Res> {
    use m::guest as x;
    Some(match fname {
        "_fold" => r(x::_fold(a.s(0, "word"))),
        "_clean" => r(x::_clean(a.s(0, "item"))),
        "_one_half" => r(x::_one_half(a.s(0, "piece"))),
        "_is_food" => r(x::_is_food(a.s(0, "item"))),
        "plain" => r(x::plain(a.s(0, "item"), a.b(1, "a_pair", false))),
        "found" => {
            let text = a.s(0, "text");
            r(x::found(text).into_iter().map(|(f, at)| json!([f, chars(text, at as usize)])).collect::<Vec<J>>())
        }
        "turned_down_alone" => r(x::turned_down_alone(a.s(0, "text"))),
        "_contains" => r(x::_contains(a.s(0, "text"), a.s(1, "food"))),
        "_without_article" => r(x::_without_article(a.s(0, "item"))),
        "_split_list" => r(x::_split_list(a.s(0, "rest"))),
        "_kept_phrase" => r(x::_kept_phrase(&a.strs(0, "items"))),
        "revise" => r(x::revise(a.s(0, "reply"), &a.v(1, "foods"))),
        "left_out" => r(x::left_out(&a.v(0, "foods"))),
        _ => return None,
    })
}

fn household(fname: &str, a: &Args) -> Option<Res> {
    use m::household as x;
    Some(match fname {
        "_who_said" => r(x::_who_said(a.s(0, "who"))),
        "_who_asked" => r(x::_who_asked(a.s(0, "who"), &a.memory(1, "memory"))),
        "_who_held" => r(x::_who_held(a.s(0, "who"), a.s(1, "key"))),
        "_the_ask_it_answers" => r(x::_the_ask_it_answers(a.s(0, "said"), &a.memory(1, "memory"))),
        "_their_households" => r(x::_their_households(&a.memory(0, "memory"))),
        "_held" => r(x::_held(&a.memory(0, "memory"))),
        "observe" => with_mem!(a, 1, "memory", |mm| x::observe(a.s(0, "text"), &mut mm)),
        "answer" => with_mem!(a, 1, "memory", |mm| x::answer(a.s(0, "text"), &mut mm)),
        "takes_as_it_comes" => r(x::takes_as_it_comes(a.s(0, "text"), &a.memory(1, "memory"))),
        _ => return None,
    })
}

fn matters(fname: &str, a: &Args) -> Option<Res> {
    use m::matters as x;
    Some(match fname {
        "_the_person" => r(x::_the_person(&a.memory(0, "memory")).map(|(w, r, s)| json!([w, r.to_plain(), s]))),
        "about_them" => r(x::about_them(a.s(0, "text"), &a.memory(1, "memory"))),
        "_a_worry_in_the_chat" => r(x::_a_worry_in_the_chat(&a.memory(0, "memory"))),
        "what_they_say" => with_mem!(a, 1, "memory", |mm| x::what_they_say(a.s(0, "text"), &mut mm)),
        "_say_the_ages" => r(x::_say_the_ages(&a.vals(0, "ages").iter().filter_map(Value::as_int).collect::<Vec<_>>())),
        "a_corrected_age" => with_mem!(a, 1, "memory", |mm| x::a_corrected_age(a.s(0, "text"), &mut mm)),
        _ => return None,
    })
}

fn rates(fname: &str, a: &Args) -> Option<Res> {
    use m::rates as x;
    Some(match fname {
        "_the_rates" => r(x::_the_rates(&a.memory(0, "memory"))),
        "_keep_a_rate" => with_mem!(a, 0, "memory", |mm| x::_keep_a_rate(&mut mm, a.s(1, "what"), a.s(2, "how"), a.s(3, "said"), a.b(4, "own", true))),
        "_the_newest_recent_rate" => r(x::_the_newest_recent_rate(&a.memory(0, "memory"), a.n_or(1, "within", 2))),
        "_the_rate_put_right" => with_mem!(a, 1, "memory", |mm| x::_the_rate_put_right(a.s(0, "text"), &mut mm)),
        "keep" => with_mem!(a, 1, "memory", |mm| x::keep(a.s(0, "text"), &mut mm)),
        "is_a_rate_ask" => r(x::is_a_rate_ask(a.s(0, "text"), &a.memory(1, "memory"))),
        "answer" => r(x::answer(a.s(0, "text"), &a.memory(1, "memory"))),
        _ => return None,
    })
}
