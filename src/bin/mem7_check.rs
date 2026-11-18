//! Check the MEM7 ports (35 files of `src/memory/`) function by function against the calls PyBot
//! made (`harness/mem7_record.py` output).
//!
//!     RSBOT_TODAY=2026-11-17 mem7_check CALLS.jsonl.gz [--fn SUBSTR] [--show K]
//!
//! Per function: calls that give PyBot's result (and leave the arguments as PyBot left them),
//! calls that differ, calls that reached a function another workstream has not ported yet
//! (UNPORTED), calls whose arguments the checker does not rebuild (a regex match, a generator)
//! and calls of functions the checker has no entry for.
#![allow(clippy::all)]
use rsbot::data::Value;
use rsbot::py::random::{self, Random};
use rsbot::py::text as pt;
use rsbot::state::Memory;
use serde_json::{json, Value as J};
use std::collections::BTreeMap;
use std::io::BufRead;

#[path = "mem7_check/calls.rs"]
mod calls;

/// Why a call was not compared.
pub enum Skip {
    Shape,
}

/// What the port gave: the result, and the arguments it may have changed (by position).
pub struct Out {
    pub result: J,
    pub after: Vec<(usize, J)>,
}

pub type Res = Result<Out, Skip>;

/// A recorded call's arguments, read positionally or by keyword.
pub struct Args<'a> {
    pub pos: &'a [J],
    pub kw: &'a serde_json::Map<String, J>,
}

impl<'a> Args<'a> {
    pub fn get(&self, i: usize, name: &str) -> Option<&'a J> {
        self.pos.get(i).or_else(|| self.kw.get(name))
    }
    pub fn has(&self, i: usize, name: &str) -> bool {
        self.get(i, name).is_some()
    }
    /// A str argument (None and anything else read as "", as `str(x or "")` does for None).
    pub fn s(&self, i: usize, name: &str) -> &'a str {
        self.get(i, name).and_then(J::as_str).unwrap_or("")
    }
    /// A str argument that has to be a str: anything else is a shape the port's `&str` cannot take.
    pub fn st(&self, i: usize, name: &str) -> Result<&'a str, Skip> {
        self.get(i, name).and_then(J::as_str).ok_or(Skip::Shape)
    }
    pub fn opt_s(&self, i: usize, name: &str) -> Option<&'a str> {
        self.get(i, name).and_then(J::as_str)
    }
    pub fn b(&self, i: usize, name: &str, default: bool) -> bool {
        match self.get(i, name) {
            Some(J::Bool(b)) => *b,
            Some(J::Null) => false,
            None => default,
            Some(J::String(s)) => !s.is_empty(),
            Some(J::Number(n)) => n.as_f64() != Some(0.0),
            Some(J::Array(a)) => !a.is_empty(),
            Some(J::Object(o)) => !o.is_empty(),
        }
    }
    pub fn n(&self, i: usize, name: &str) -> Result<i64, Skip> {
        self.get(i, name).and_then(J::as_i64).ok_or(Skip::Shape)
    }
    pub fn n_or(&self, i: usize, name: &str, default: i64) -> i64 {
        self.get(i, name).and_then(J::as_i64).unwrap_or(default)
    }
    pub fn v(&self, i: usize, name: &str) -> Value {
        self.get(i, name).map(Value::from_plain).unwrap_or(Value::None)
    }
    pub fn strs(&self, i: usize, name: &str) -> Vec<String> {
        self.get(i, name)
            .and_then(J::as_array)
            .map(|a| a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect())
            .unwrap_or_default()
    }
    pub fn vals(&self, i: usize, name: &str) -> Vec<Value> {
        self.v(i, name).items().to_vec()
    }
    /// A Python position (code points) in `text` as a byte offset.
    pub fn pos_in(&self, text: &str, i: usize, name: &str) -> Result<usize, Skip> {
        let n = self.n(i, name)?;
        if n < 0 {
            return Err(Skip::Shape);
        }
        Ok(pt::byte_at(text, n as usize))
    }
    pub fn memory(&self, i: usize, name: &str) -> Memory {
        match self.get(i, name) {
            Some(J::Object(m)) => Memory::from_plain(m),
            _ => Memory::new(),
        }
    }
    /// A memory argument that may be None.
    pub fn opt_memory(&self, i: usize, name: &str) -> Option<Memory> {
        match self.get(i, name) {
            Some(J::Object(m)) => Some(Memory::from_plain(m)),
            _ => None,
        }
    }
    /// A function argument's Python id ("module:qualname:line").
    pub fn func(&self, i: usize, name: &str) -> Option<&'a str> {
        self.get(i, name).and_then(|j| j.get("__fn__")).and_then(J::as_str)
    }
}

fn has_tag(j: &J) -> bool {
    match j {
        J::Object(o) if o.contains_key("__re__") || o.contains_key("__fn__") => false,
        J::Object(o) => o.contains_key("__match__") || o.contains_key("__opaque__") || o.values().any(has_tag),
        J::Array(a) => a.iter().any(has_tag),
        _ => false,
    }
}

pub trait ToJ {
    fn j(self) -> J;
}
impl ToJ for J {
    fn j(self) -> J {
        self
    }
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
impl ToJ for usize {
    fn j(self) -> J {
        json!(self)
    }
}
impl ToJ for f64 {
    fn j(self) -> J {
        json!(self)
    }
}
impl ToJ for () {
    fn j(self) -> J {
        J::Null
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
impl<A: ToJ, B: ToJ, C: ToJ, D: ToJ> ToJ for (A, B, C, D) {
    fn j(self) -> J {
        J::Array(vec![self.0.j(), self.1.j(), self.2.j(), self.3.j()])
    }
}

/// A result with no argument changed.
pub fn r<T: ToJ>(x: T) -> Res {
    Ok(Out { result: x.j(), after: Vec::new() })
}

/// A result and the memory argument (at `index`) as the call left it.
pub fn rm<T: ToJ>(x: T, index: usize, memory: &Memory) -> Res {
    Ok(Out { result: x.j(), after: vec![(index, J::Object(memory.to_plain()))] })
}

/// A result and a value argument (at `index`) as the call left it.
pub fn rv<T: ToJ>(x: T, index: usize, value: &Value) -> Res {
    Ok(Out { result: x.j(), after: vec![(index, value.to_plain())] })
}

/// Memory is compared without the reading (`understood` is typed in Rust and a dict in Python).
fn comparable(mut j: J) -> J {
    if let Some(o) = j.as_object_mut() {
        if o.contains_key("turns") || o.contains_key("understood") {
            o.remove("understood");
            if o.get("turns").and_then(J::as_array).is_some_and(|t| t.is_empty()) {
                o.remove("turns");
            }
        }
    }
    j
}

/// Ints and floats that are the same number compare equal, as in Python.
fn same(a: &J, b: &J) -> bool {
    match (a, b) {
        (J::Number(x), J::Number(y)) => x.as_f64() == y.as_f64(),
        (J::Array(x), J::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q)),
        (J::Object(x), J::Object(y)) => x.len() == y.len() && x.iter().all(|(k, p)| y.get(k).is_some_and(|q| same(p, q))),
        _ => a == b,
    }
}

fn main() {
    rsbot::runtime::quiet_panics();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = args.first().expect("a recording from harness/mem7_record.py");
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
    let mut unported: BTreeMap<String, usize> = BTreeMap::new();
    let empty = serde_json::Map::new();
    for line in reader.lines() {
        let Ok(line) = line else { break };
        let row: J = serde_json::from_str(&line).unwrap();
        if row.get("dep").is_some() {
            continue; // a call of another workstream's function, kept for the mock crate
        }
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
        match row.get("rng").and_then(J::as_array) {
            Some(state) => {
                let words: Vec<u64> = state.iter().map(|x| x.as_u64().unwrap()).collect();
                random::setstate(Random::from_python_state(&words));
            }
            None => random::seed(0),
        }
        let got = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| calls::call(&name, &a)));
        match got {
            Ok(None) => t[4] += 1,
            Ok(Some(Err(Skip::Shape))) => t[3] += 1,
            Ok(Some(Ok(out))) => {
                let want = calls::expected(&row, &name);
                let mut ok = same(&out.result, &want);
                let mut why = String::new();
                for (index, after) in &out.after {
                    let want_after = row
                        .get("after")
                        .and_then(|x| x.get(*index))
                        .or_else(|| row["args"].get(*index))
                        .cloned()
                        .unwrap_or(J::Null);
                    let (got_after, want_after) = (comparable(after.clone()), comparable(want_after));
                    if !same(&got_after, &want_after) {
                        ok = false;
                        why = format!("\n  arg {index} after, python: {want_after}\n  arg {index} after, rust:   {got_after}");
                    }
                }
                if ok {
                    t[0] += 1;
                } else {
                    t[1] += 1;
                    let n = shown.entry(name.clone()).or_default();
                    if *n < show {
                        *n += 1;
                        println!("DIFF {name} args={} kwargs={}\n  python: {}\n  rust:   {}{why}", row["args"], row["kwargs"], want, out.result);
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
                    *unported.entry(msg.replace("UNPORTED ", "")).or_default() += 1;
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
    if !unported.is_empty() {
        println!("\nunported functions reached:");
        for (name, n) in &unported {
            println!("  {n:>6} {name}");
        }
    }
}
