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
