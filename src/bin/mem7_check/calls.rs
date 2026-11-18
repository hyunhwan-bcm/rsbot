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
