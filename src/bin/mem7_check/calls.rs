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
