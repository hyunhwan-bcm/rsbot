//! Port of PyBot's `bot/understanding` package (`bot/understanding/__init__.py` is this file).
//! Each Python file `bot/understanding/<name>.py` is `<name>.rs` here.
//!
//! `understand(text, memory)` is the reading every module looks at first.

use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub mod attempts;
pub mod body;
pub mod breeds;
pub mod car_glass;
pub mod cars;
pub mod confusion;
pub mod kinds;
pub mod lexicon;
pub mod normalize;
pub mod references;
pub mod senses;
pub mod topics;

/// A regex built from a pattern made at run time (`re.search(r"\b" + re.escape(cue) ...)`),
/// compiled once per distinct pattern and kept for the life of the process (the patterns come
/// from PyBot's own tables, so there are finitely many).
pub fn rx(pattern: &str) -> &'static Re {
    rx_flags(pattern, 0)
}

/// [`rx`] with Python flag bits (`re::I`, ...).
pub fn rx_flags(pattern: &str, flags: u32) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<(String, u32), &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let key = (pattern.to_string(), flags);
    if let Some(r) = CACHE.lock().unwrap().get(&key) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(pattern, flags)));
    CACHE.lock().unwrap().insert(key, r);
    r
}

/// `understand(text, memory=None)`: the reading of one line, read against the conversation so
/// far. With `RSBOT_ORACLE_UNDERSTAND` set, PyBot's own recorded reading is returned for a text
/// it has seen (for checking modules before this engine is complete).
pub fn understand(text: &str, memory: Option<&Memory>) -> Understood {
    if crate::oracle::use_oracle_understand() {
        if let Some(u) = crate::oracle::UNDERSTAND_CALLS.get(text) {
            return u.clone();
        }
    }
    let _ = memory;
    todo!("understanding::understand")
}
