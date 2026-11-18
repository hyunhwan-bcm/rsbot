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
        "lineback" => lineback(fname, a),
        "meant" => meant(fname, a),
        "obstacle" => obstacle(fname, a),
        "plan" => plan(fname, a),
        "plans" => plans(fname, a),
        "portions" => portions(fname, a),
        "reject" => reject(fname, a),
        "resize" => resize(fname, a),
        "stands" => stands(fname, a),
        "surname" => surname(fname, a),
        "taking" => taking(fname, a),
        "years_in" => years_in(fname, a),
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
        "_the_period_at" => {
            let text = a.s(0, "text");
            a.pos_in(text, 1, "at").and_then(|at| r(x::_the_period_at(text, at)))
        }
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
        "_the_line_the_ask_names" => {
            let (kind, named, found) = x::_the_line_the_ask_names(a.s(0, "text"), &a.memory(1, "memory"));
            r(json!([kind, named, said_j(found)]))
        }
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
        "_the_group_ages" => r(match x::_the_group_ages(&a.memory(0, "memory")) {
            Some((index, ages)) => {
                let entry = a.memory(0, "memory").get("others").map(|o| o.items()[index].to_plain()).unwrap_or(J::Null);
                json!([entry, ages])
            }
            None => json!([null, []]),
        }),
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

fn set_of(a: &Args, i: usize, name: &str) -> std::collections::HashSet<String> {
    a.strs(i, name).into_iter().collect()
}

fn lineback(fname: &str, a: &Args) -> Option<Res> {
    use m::lineback as x;
    Some(match fname {
        "flat" => r(x::flat(a.s(0, "text"))),
        "_words" => r(x::_words(a.s(0, "text"))),
        "_stem" => r(x::_stem(a.s(0, "word"))),
        "about" => r(sorted_set(x::about(a.s(0, "text")))),
        "about_any" => r(sorted_set(x::about_any(a.s(0, "text")))),
        "_a_kind_named" => r(x::_a_kind_named(a.s(0, "text"))),
        "is_a_recall_ask" => r(x::is_a_recall_ask(a.s(0, "text"))),
        "_the_last_reply" => r(x::_the_last_reply(&a.memory(0, "memory"))),
        "the_lines" => r(x::the_lines(&a.memory(0, "memory"))),
        "_the_best_line" => r(x::_the_best_line(&set_of(a, 0, "wanted"), &a.strs(1, "lines"), a.b(2, "questions", false))),
        "_the_kind_asked" => r(x::_the_kind_asked(a.s(0, "said"))),
        "_a_value_of_the_kind" => r(x::_a_value_of_the_kind(a.s(0, "kind"), &a.strs(1, "lines"))),
        "_the_value" => r(x::_the_value(a.s(0, "line"))),
        "_carried" => r(x::_carried(a.s(0, "line"), &set_of(a, 1, "wanted"))),
        "_the_value_of_the_kind" => r(x::_the_value_of_the_kind(a.s(0, "said"), &a.strs(1, "lines"), &a.memory(2, "memory"))),
        "_the_figure_for_a_thing" => r(x::_the_figure_for_a_thing(a.s(0, "said"), &a.strs(1, "lines"), &a.memory(2, "memory"))),
        "_the_line_itself" => r(x::_the_line_itself(a.s(0, "said"), &a.strs(1, "lines"), &a.memory(2, "memory"))),
        "_the_kind_they_gave" => r(x::_the_kind_they_gave(a.s(0, "said"), &a.strs(1, "lines"), &a.memory(2, "memory"))),
        "the_kind_they_gave" => r(x::the_kind_they_gave(a.s(0, "text"), &a.memory(1, "memory"))),
        "an_earlier_turn" => r(x::an_earlier_turn(a.s(0, "text"), &a.memory(1, "memory"))),
        "reply_for" => r(x::reply_for(a.s(0, "text"), &a.memory(1, "memory"))),
        _ => return None,
    })
}

fn meant(fname: &str, a: &Args) -> Option<Res> {
    use m::meant as x;
    Some(match fname {
        "_asked_plainly" => r(x::_asked_plainly(a.s(0, "clean"))),
        "_personal" => r(x::_personal(a.s(0, "said"))),
        "_their_subject" => r(x::_their_subject(a.s(0, "said"))),
        "_on_the_table" => r(x::_on_the_table(a.s(0, "noun"), &a.memory(1, "memory"))),
        "_under_the_lead" => r(x::_under_the_lead(a.s(0, "said"))),
        "_sentences" => r(x::_sentences(a.s(0, "said"))),
        "_gist" => r(x::_gist(a.s(0, "sentence"))),
        "_answers" => r(x::_answers(&a.memory(0, "memory"), a.n_or(1, "back", 3) as usize)),
        "_thick_enough" => r(x::_thick_enough(a.s(0, "said"), a.s(1, "asked"))),
        "_framed" => r(x::_framed(a.s(0, "said"))),
        "_a_figure" => r(x::_a_figure(a.s(0, "said"))),
        "_a_question" => r(x::_a_question(a.s(0, "sentence"))),
        "_worth_handing_over" => r(x::_worth_handing_over(a.s(0, "said"), a.s(1, "sentence"), a.s(2, "asked"))),
        "_cite" => r(x::_cite(a.s(0, "said"), a.s(1, "sentence"), a.s(2, "asked"), a.opt_memory(3, "memory").as_ref())),
        "_meant" => r(x::_meant(a.s(0, "said"), a.b(1, "asked_what_it_meant", false))),
        "_after_the_condolence" => r(x::_after_the_condolence(a.s(0, "said"))),
        "_same_bit" => r(x::_same_bit(a.s(0, "one"), a.s(1, "other"))),
        "_already" => r(x::_already(a.s(0, "cited"), &a.strs(1, "said_since"))),
        "_carried" => r(x::_carried(a.s(0, "said"))),
        "_applied_to_the_person" => with_mem!(a, 3, "memory", |mm| x::_applied_to_the_person(a.s(0, "clean"), a.s(1, "said"), a.s(2, "asked"), &mut mm, a.b(4, "what_it_meant", false))),
        "answer_for" => {
            let mut mm = a.memory(1, "memory");
            // The reading is the memory's own (`memory["understood"]`).
            let out = x::answer_for(a.s(0, "text"), &mut mm);
            rm(out, 1, &mm)
        }
        "reads_it" => r(x::reads_it(a.s(0, "text"))),
        _ => return None,
    })
}

fn obstacle(fname: &str, a: &Args) -> Option<Res> {
    use m::obstacle as x;
    Some(match fname {
        "_flat" => r(x::_flat(a.s(0, "text"))),
        "_words" => r(x::_words(a.s(0, "text"))),
        "the_thing" => r(x::the_thing(a.s(0, "text"))),
        "the_thing_they_cannot_use" => r(x::the_thing_they_cannot_use(a.s(0, "text"))),
        "the_move" => r(x::the_move(a.s(0, "text"))),
        "_the_family" => r(x::_the_family(a.s(0, "move"))),
        "_opens_on_a_verb" => r(x::_opens_on_a_verb(a.s(0, "move"))),
        "_the_noun" => r(x::_the_noun(a.s(0, "move"))),
        "_a_list_under_a_frame" => r(x::_a_list_under_a_frame(a.s(0, "one"))),
        "_a_doing" => r(x::_a_doing(a.s(0, "one"))),
        "_a_whole_step" => r(x::_a_whole_step(a.s(0, "sentence"))),
        "a_move_refused" => with_mem!(a, 1, "memory", |mm| x::a_move_refused(a.s(0, "text"), &mut mm)),
        "_under_its_frame" => r(x::_under_its_frame(a.s(0, "one"))),
        "_the_frame" => r(x::_the_frame(a.s(0, "one"))),
        "_steps" => r(x::_steps(a.s(0, "said")).into_iter().map(|(s, p, u)| json!([s, p, u])).collect::<Vec<J>>()),
        "_needs_it" => r(x::_needs_it(a.s(0, "step"), a.s(1, "thing"))),
        "_leans_on_it" => r(x::_leans_on_it(a.s(0, "said"), a.s(1, "thing"), a.s(2, "asked"))),
        "answer_for" => with_mem!(a, 1, "memory", |mm| x::answer_for(a.s(0, "text"), &mut mm)),
        "answer_for_a_thing_they_cannot_use" => with_mem!(a, 1, "memory", |mm| x::answer_for_a_thing_they_cannot_use(a.s(0, "text"), &mut mm)),
        _ => return None,
    })
}

fn plan(fname: &str, a: &Args) -> Option<Res> {
    use m::plan as x;
    Some(match fname {
        "_the_thing_named" => r(sorted_set(x::_the_thing_named(a.s(0, "text")))),
        "_tried" => r(x::_tried(a.s(0, "step"), &a.memory(1, "memory"))),
        "the_last_answer" => r(x::the_last_answer(&a.memory(0, "memory"))),
        "the_answer_in_hand" => r(x::the_answer_in_hand(&a.memory(0, "memory"))),
        "_a_cut_at_a_colon" => r(x::_a_cut_at_a_colon(a.s(0, "step"))),
        "steps_of" => r(x::steps_of(a.s(0, "answer"))),
        "_words" => r(sorted_set(x::_words(a.s(0, "text")))),
        "_stems" => r(sorted_set(x::_stems(a.s(0, "text")))),
        "carry_on" => with_mem!(a, 1, "memory", |mm| x::carry_on(a.s(0, "text"), &mut mm)),
        _ => return None,
    })
}

fn plans(fname: &str, a: &Args) -> Option<Res> {
    use m::plans as x;
    Some(match fname {
        "_a_first_day" => r(x::_a_first_day(a.s(0, "text"))),
        "as_question" => r(x::as_question(a.s(0, "text"))),
        "_implied" => r(x::_implied(a.s(0, "text"))),
        "answer" => with_mem!(a, 1, "memory", |mm| x::answer(a.s(0, "text"), &mut mm)),
        "_the_clauses" => r(x::_the_clauses(a.s(0, "detail"))),
        "_the_first_step" => r(x::_the_first_step(a.s(0, "sentence"))),
        "_part_words" => r(sorted_set(x::_part_words(a.s(0, "text")))),
        "_was_a_plan" => r(x::_was_a_plan(a.s(0, "line"))),
        "follow_up" => r(x::follow_up(a.s(0, "text"), &a.memory(1, "memory"))),
        _ => return None,
    })
}

fn portions(fname: &str, a: &Args) -> Option<Res> {
    use m::portions as x;
    Some(match fname {
        "_the_number" => r(x::_the_number(a.s(0, "word"))),
        "_as_a_count" => r(x::_as_a_count(a.s(0, "word"))),
        "_the_per_head_figure" => r(x::_the_per_head_figure(a.s(0, "said"))),
        "the_portion_answer" => r(x::the_portion_answer(&a.memory(0, "memory"))),
        "an_ask" => r(x::an_ask(a.s(0, "text"), &a.memory(1, "memory"))),
        "reply_for" => with_mem!(a, 1, "memory", |mm| x::reply_for(a.s(0, "text"), &mut mm)),
        "_the_new_count" => {
            // Python returns the Match (encoded opaque); compare whether there is one.
            return Some(Err(Skip::Shape));
        }
        "_the_count_they_gave" => r(x::_the_count_they_gave(a.s(0, "text"))),
        "_a_total_for_a_bare_count" => r(x::_a_total_for_a_bare_count(a.s(0, "text"), &a.memory(1, "memory"))),
        "a_bare_count_to_fill" => r(x::a_bare_count_to_fill(a.s(0, "text"), &a.memory(1, "memory"))),
        "a_count_to_fill" => r(x::a_count_to_fill(a.s(0, "text"), &a.memory(1, "memory"))),
        "the_total_for" => with_mem!(a, 1, "memory", |mm| x::the_total_for(a.s(0, "text"), &mut mm)),
        _ => return None,
    })
}

fn reject(fname: &str, a: &Args) -> Option<Res> {
    use m::reject as x;
    Some(match fname {
        "_words" => r(x::_words(a.s(0, "text"))),
        "refusal" => r(x::refusal(a.s(0, "text"), a.s(1, "offer"))),
        "_the_whole_name" => r(x::_the_whole_name(a.s(0, "option"), a.s(1, "reply"))),
        "_tidy" => r(x::_tidy(a.s(0, "option"))),
        "_by_title" => r(x::_by_title(&a.strs(0, "titles"), &a.strs(1, "options"))),
        "_a_choice_of_things" => r(x::_a_choice_of_things(a.s(0, "reply"))),
        "_the_offer_reply" => r(x::_the_offer_reply(&a.memory(0, "memory"))),
        "_options" => r(x::_options(a.s(0, "reply"))),
        "_a_list_of_causes" => r(x::_a_list_of_causes(a.s(0, "reply"), &a.strs(1, "options"))),
        "_a_noun_phrase" => r(x::_a_noun_phrase(a.s(0, "option"))),
        "_said_by_them" => r(x::_said_by_them(a.s(0, "option"), a.s(1, "text"))),
        "_offered" => r(x::_offered(&a.memory(0, "memory"))),
        "_the_fence" => r(x::_the_fence(&a.memory(0, "memory"))),
        "_the_whole_lot" => r(x::_the_whole_lot(&a.strs(0, "options"))),
        "offers_a_choice" => r(x::offers_a_choice(&a.memory(0, "memory"))),
        "_the_names" => r(x::_the_names(&a.strs(0, "options"))),
        "_the_lot_and_the_verb" => r(x::_the_lot_and_the_verb(&a.strs(0, "lines"))),
        "reply_for" => r(x::reply_for(a.s(0, "text"), &a.memory(1, "memory"))),
        _ => return None,
    })
}

fn figure_j(found: Option<(f64, String, (usize, usize), String)>, text: &str) -> J {
    match found {
        Some((v, u, (s, e), f)) => json!([v, u, [chars(text, s), chars(text, e)], f]),
        None => J::Null,
    }
}

fn resize(fname: &str, a: &Args) -> Option<Res> {
    use m::resize as x;
    let squashed = |t: &str| pt::lower(&pt::squash(t));
    Some(match fname {
        "quantity" => r(figure_j(x::quantity(a.s(0, "text")), &squashed(a.s(0, "text")))),
        "rate" => r(figure_j(x::rate(a.s(0, "text")), &squashed(a.s(0, "text")))),
        "_earlier_lines" => r(x::_earlier_lines(&a.v(0, "memory"), a.n_or(1, "n", 4) as usize)),
        "_a_bare_rate" => r(x::_a_bare_rate(a.s(0, "text"))),
        "_with_a_new_rate" => match a.get(1, "value").and_then(J::as_f64) {
            Some(v) => r(x::_with_a_new_rate(a.s(0, "said"), v, a.opt_s(2, "unit"), &a.v(3, "memory"))),
            None => Err(Skip::Shape),
        },
        "_value" => r(x::_value(a.s(0, "raw"))),
        "headcount" => {
            let said = squashed(a.s(0, "text"));
            r(x::headcount(a.s(0, "text")).map(|(v, s, e)| json!([v, chars(&said, s), chars(&said, e)])))
        }
        "_a_question" => r(x::_a_question(a.s(0, "earlier"))),
        "rewritten" => r(x::rewritten(a.s(0, "text"), &a.v(1, "memory"))),
        _ => return None,
    })
}

fn stands(fname: &str, a: &Args) -> Option<Res> {
    use m::stands as x;
    Some(match fname {
        "_number" => r(x::_number(a.s(0, "word"))),
        "_figures" => {
            let text = a.s(0, "text");
            r(x::_figures(text).into_iter().map(|(v, u, s, e)| json!([v, u, chars(text, s), chars(text, e)])).collect::<Vec<J>>())
        }
        "_the_line_asks_about_a_figure_and_elsewhere_nothing" => r(x::_the_line_asks_about_a_figure_and_elsewhere_nothing(a.s(0, "text")).map(|(v, u, p)| json!([v, u, p]))),
        "the_figure_asked_about" => r(x::the_figure_asked_about(a.s(0, "text"), &a.memory(1, "memory")).map(|(v, u, p, q)| json!([v, u, p, q]))),
        "_a_change_was_made" => r(x::_a_change_was_made(&a.memory(0, "memory"))),
        "_a_change_of_size" => r(x::_a_change_of_size(&a.memory(0, "memory"))),
        "the_change_asked_about" => r(x::the_change_asked_about(a.s(0, "text"), &a.memory(1, "memory"))),
        "change_reply" => r(x::change_reply(a.s(0, "text"), &a.memory(1, "memory"))),
        "reply_for" => r(x::reply_for(a.s(0, "text"), &a.memory(1, "memory"))),
        _ => return None,
    })
}

fn surname(fname: &str, a: &Args) -> Option<Res> {
    use m::surname as x;
    Some(match fname {
        "_the_name_they_said" => r(x::_the_name_they_said(a.s(0, "word"))),
        "the_surname" => r(x::the_surname(&a.memory(0, "memory"))),
        "_given_one" => r(x::_given_one(a.s(0, "text"))),
        "observe" => with_mem!(a, 1, "memory", |mm| x::observe(a.s(0, "text"), &mut mm)),
        "_asks_for_it" => r(x::_asks_for_it(a.s(0, "text"))),
        "a_note" => r(x::a_note(a.s(0, "text"), &a.memory(1, "memory"))),
        "a_follow_up" => r(x::a_follow_up(a.s(0, "text"), &a.memory(1, "memory"))),
        "there_is_a_chat" => r(x::there_is_a_chat(&a.memory(0, "memory"))),
        "reply_for" => with_mem!(a, 1, "memory", |mm| x::reply_for(a.s(0, "text"), &mut mm)),
        "_plain" => r(x::_plain(a.s(0, "text"))),
        _ => return None,
    })
}

fn taking(fname: &str, a: &Args) -> Option<Res> {
    use m::taking as x;
    Some(match fname {
        "_flat" => r(x::_flat(a.s(0, "text"))),
        "_asks_what_to_take" => r(x::_asks_what_to_take(a.s(0, "text"))),
        "_a_complaint" => r(x::_a_complaint(a.s(0, "subject"))),
        "_subjects" => r(x::_subjects(&a.memory(0, "memory"))),
        "_the_things_to_take_for" => r(x::_the_things_to_take_for(a.s(0, "subject"))),
        "_a_trip_above" => r(x::_a_trip_above(&a.memory(0, "memory"), a.s(1, "text"))),
        "the_ask_and_the_subject" => r(x::the_ask_and_the_subject(a.s(0, "text"), &a.memory(1, "memory")).unwrap_or(false)),
        "reply_for" => r(x::reply_for(a.s(0, "text"), &a.memory(1, "memory"))),
        _ => return None,
    })
}

fn years_in(fname: &str, a: &Args) -> Option<Res> {
    use m::years_in as x;
    Some(match fname {
        "_a_job_word" => r(x::_a_job_word(a.s(0, "doing"))),
        "_alike" => r(x::_alike(a.s(0, "doing"), a.s(1, "job"))),
        "_the_filed_job" => r(x::_the_filed_job(&a.memory(0, "memory"))),
        "_the_doing_and_the_length" => r(x::_the_doing_and_the_length(a.s(0, "text"), &a.memory(1, "memory"))),
        "_the_count" => r(x::_the_count(a.s(0, "word"))),
        "_the_figure_in_words" => a.n(0, "count").and_then(|c| r(x::_the_figure_in_words(c))),
        "the_years_in_the_doing" => r(x::the_years_in_the_doing(a.s(0, "text"), &a.memory(1, "memory"))),
        "the_years_in_their_doing" => r(x::the_years_in_their_doing(a.s(0, "text"), &a.memory(1, "memory"))),
        _ => return None,
    })
}
