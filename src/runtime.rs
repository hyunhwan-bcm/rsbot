//! One turn: PyBot's `bot/runtime.py`, for the hot-path modules this port carries.
//!
//! Observers run first (highest PRIORITY first), then every module proposes a reply on its own
//! copy of memory, and the arbiter picks one; the chosen module's copy becomes the memory. A
//! module that fails is skipped and a turn never fails, as in Python: a Rust panic inside a
//! module stands for a Python exception and is caught here.

use crate::state::{Memory, Turn};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Once;

pub type Observe = fn(&str, &mut Memory);
pub type Respond = fn(&str, &mut Memory) -> Option<String>;
pub type After = fn(&str, &str, &mut Memory);

pub struct ModuleDef {
    pub name: &'static str,
    pub priority: f64,
    pub observe: Option<Observe>,
    pub respond: Option<Respond>,
    pub after: Option<After>,
}

#[derive(Clone, Debug)]
pub struct Candidate {
    pub module: &'static str,
    pub priority: f64,
    pub reply: String,
}

#[derive(Clone, Debug)]
pub struct TurnResult {
    pub reply: String,
    pub module: Option<&'static str>,
    pub candidates: Vec<Candidate>,
}

pub const FALLBACK: &str = "Hello!";

static QUIET: Once = Once::new();

thread_local! {
    static LAST_PANIC: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// Panics inside modules are expected (they stand for Python exceptions); keep them off stderr
/// unless RSBOT_PANICS is set. The last panic's message is kept for the checkers
/// ([`take_last_panic`]), so an `unported!` stub can be told from a real failure.
pub fn quiet_panics() {
    QUIET.call_once(|| {
        let loud = std::env::var_os("RSBOT_PANICS").is_some();
        std::panic::set_hook(Box::new(move |info| {
            let msg = info
                .payload()
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| info.payload().downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default();
            if loud {
                eprintln!("panic: {msg} at {:?}", info.location());
            }
            LAST_PANIC.with(|p| *p.borrow_mut() = Some(msg));
        }));
    });
}

/// The message of the last panic on this thread (and forget it).
pub fn take_last_panic() -> Option<String> {
    LAST_PANIC.with(|p| p.borrow_mut().take())
}

/// Run one module hook, turning a panic into "this module passed".
pub fn guarded<T>(f: impl FnOnce() -> T) -> Option<T> {
    catch_unwind(AssertUnwindSafe(f)).ok()
}

/// The arbiter's pick as an index; 0 when it fails or there is nothing to choose between.
fn choose(text: &str, memory: &Memory, candidates: &[Candidate]) -> usize {
    if candidates.len() < 2 {
        return 0;
    }
    match guarded(|| crate::modules::choose(text, memory, candidates)) {
        Some(i) if i < candidates.len() => i,
        _ => 0,
    }
}

/// One turn: the reply, the module that gave it, and every candidate. Updates memory in place.
pub fn turn(text: &str, memory: &mut Memory) -> TurnResult {
    quiet_panics();
    let modules = crate::modules::registry();
    for m in modules {
        if let Some(observe) = m.observe {
            let _ = guarded(|| observe(text, memory));
        }
    }
    let mut candidates = Vec::new();
    let mut copies = Vec::new();
    for m in modules {
        let Some(respond) = m.respond else { continue };
        let mut mine = memory.clone();
        if let Some(Some(answer)) = guarded(|| respond(text, &mut mine)) {
            if !crate::py::text::strip(&answer).is_empty() {
                candidates.push(Candidate { module: m.name, priority: m.priority, reply: answer });
                copies.push(mine);
            }
        }
    }
    let (reply, module) = if candidates.is_empty() {
        (FALLBACK.to_string(), None)
    } else {
        let i = choose(text, memory, &candidates);
        *memory = copies.swap_remove(i);
        (candidates[i].reply.clone(), Some(candidates[i].module))
    };
    for m in modules {
        if let Some(after) = m.after {
            let _ = guarded(|| after(text, &reply, memory));
        }
    }
    memory.turns.push(Turn { user: text.to_string(), bot: reply.clone(), module: module.map(str::to_string) });
    TurnResult { reply, module, candidates }
}

/// `pybot.py`'s one-line answer: whitespace collapsed, "Hello!" if nothing is left.
pub fn answer(text: &str, memory: &mut Memory) -> (String, Option<&'static str>) {
    let r = turn(text, memory);
    let reply = crate::py::text::squash(&r.reply);
    (if reply.is_empty() { FALLBACK.to_string() } else { reply }, r.module)
}
